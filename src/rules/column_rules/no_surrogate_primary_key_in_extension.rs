//! Submodule providing the `NoSurrogatePrimaryKeyInExtension` constraint,
//! which enforces that primary-key columns in extension tables are not
//! surrogate keys.

use sql_traits::traits::{ColumnLike, DatabaseLike, TableLike};

use crate::{
    error::RuleErrorInfo,
    traits::{ColumnRule, Constrainer, GenericConstrainer},
};

/// Struct defining a constraint that enforces that if a table is not a root
/// table (i.e. it extends another table), then any primary-key column in that
/// table must not be surrogate.
///
/// This rule treats generated columns (e.g. `SERIAL`, `AUTOINCREMENT`) and
/// columns with `DEFAULT` values as surrogate-like primary keys for extension
/// tables.
///
/// # Example
///
/// Here follows an example of validating schemas with the
/// `NoSurrogatePrimaryKeyInExtension` constraint.
///
/// ```rust
/// use sql_rules::prelude::*;
///
/// let constrainer: GenericConstrainer<ParserDB> =
///     NoSurrogatePrimaryKeyInExtension::default().into();
///
/// let invalid_schema = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE root_entities (id INT PRIMARY KEY);
/// CREATE TABLE child_entities (
///     id SERIAL PRIMARY KEY REFERENCES root_entities(id)
/// );
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// let invalid_default_schema = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE root_entities (id INT PRIMARY KEY);
/// CREATE TABLE child_entities (
///     id INT PRIMARY KEY DEFAULT 42 REFERENCES root_entities(id)
/// );
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&invalid_default_schema).is_err());
///
/// let valid_schema = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE root_entities (id SERIAL PRIMARY KEY);
/// CREATE TABLE child_entities (
///     id INT PRIMARY KEY REFERENCES root_entities(id)
/// );
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct NoSurrogatePrimaryKeyInExtension<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for NoSurrogatePrimaryKeyInExtension<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<NoSurrogatePrimaryKeyInExtension<DB>>
    for GenericConstrainer<DB>
{
    fn from(constraint: NoSurrogatePrimaryKeyInExtension<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_column_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> ColumnRule for NoSurrogatePrimaryKeyInExtension<DB> {
    type Database = DB;

    fn validate_column(
        &self,
        database: &Self::Database,
        column: &<Self::Database as DatabaseLike>::Column,
    ) -> Result<(), crate::error::Error<DB>> {
        if !column.is_primary_key(database)? {
            return Ok(());
        }

        let table = column.table(database);
        if !table.is_extension(database)? {
            return Ok(());
        }

        let table_name = table.table_name();
        let column_name = column.column_name();
        let surrogate_reason = match (column.is_generated(), column.has_default()) {
            (true, true) => "is generated and defines a DEFAULT value",
            (true, false) => "is generated (e.g. SERIAL/AUTOINCREMENT)",
            (false, true) => "defines a DEFAULT value",
            (false, false) => return Ok(()),
        };

        let error: RuleErrorInfo = RuleErrorInfo::builder()
            .rule("NoSurrogatePrimaryKeyInExtension")
            .unwrap()
            .object(format!("{table_name}.{column_name}"))
            .unwrap()
            .message(format!(
                "Primary-key column '{table_name}.{column_name}' belongs to an extension table and {surrogate_reason}"
            ))
            .unwrap()
            .resolution(format!(
                "Use a non-surrogate primary key for '{table_name}.{column_name}' by removing SERIAL/AUTOINCREMENT/DEFAULT and reusing the inherited key value"
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
