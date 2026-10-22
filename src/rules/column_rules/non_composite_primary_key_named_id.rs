//! Submodule providing the `NonCompositePrimaryKeyNamedId` constraint, which
//! enforces that if a column is a non-composite primary key, it must be named
//! "id".

use sql_traits::traits::{ColumnLike, DatabaseLike, TableLike};

use crate::{
    error::RuleErrorInfo,
    traits::{ColumnRule, Constrainer, GenericConstrainer},
};

/// Struct defining a constraint that enforces that if a column is a
/// non-composite primary key, it must be named "id".
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `NonCompositePrimaryKeyNamedId` constraint.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = NonCompositePrimaryKeyNamedId::default().into();
///
/// let invalid_schema =
///     ParserDB::parse::<GenericDialect>("CREATE TABLE mytable (pk INT PRIMARY KEY, name TEXT);").unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// let valid_schema =
///     ParserDB::parse::<GenericDialect>("CREATE TABLE mytable (id INT PRIMARY KEY, name TEXT);").unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
///
/// // Composite primary keys are allowed to have any name
/// let valid_composite_schema = ParserDB::parse::<GenericDialect>(
///     "CREATE TABLE mytable (pk1 INT, pk2 INT, name TEXT, PRIMARY KEY (pk1, pk2));",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_composite_schema).is_ok());
/// ```
pub struct NonCompositePrimaryKeyNamedId<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for NonCompositePrimaryKeyNamedId<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<NonCompositePrimaryKeyNamedId<DB>>
    for GenericConstrainer<DB>
{
    fn from(constraint: NonCompositePrimaryKeyNamedId<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_column_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> ColumnRule for NonCompositePrimaryKeyNamedId<DB> {
    type Database = DB;

    fn validate_column(
        &self,
        database: &Self::Database,
        column: &<Self::Database as DatabaseLike>::Column,
    ) -> Result<(), crate::error::Error<Self::Database>> {
        // Check if this column is a primary key
        if !column.is_primary_key(database)? {
            return Ok(());
        }

        // Get the table to check if it has a composite primary key
        let table = column.table(database);
        let pk_columns: Vec<_> = table.primary_key_columns(database)?.collect();

        // If the primary key is composite, the constraint doesn't apply
        if pk_columns.len() > 1 {
            return Ok(());
        }

        // Single primary key column must be named "id"
        if column.column_name() == "id" {
            Ok(())
        } else {
            let column_name = column.column_name();
            let table_name = table.table_name();

            let error: RuleErrorInfo = RuleErrorInfo::builder()
                .rule("NonCompositePrimaryKeyNamedId")
                .unwrap()
                .object(format!("{table_name}.{column_name}"))
                .unwrap()
                .message(format!(
                    "Column '{column_name}' in table '{table_name}' is a non-composite primary key but is not named 'id'"
                ))
                .unwrap()
                .resolution(format!(
                    "Rename the primary key column '{column_name}' to 'id' in table '{table_name}'"
                ))
                .unwrap()
                .try_into()
                .unwrap();
            Err(crate::error::Error::Column(
                Box::new(column.clone()),
                error.into(),
            ))
        }
    }
}
