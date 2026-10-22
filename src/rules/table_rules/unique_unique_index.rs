//! Submodule providing the `UniqueUniqueIndex` constraint, which enforces
//! that check constraints are unique per table, meaning that no two check
//! constraints have the same clause in a single table.

use crate::{
    error::RuleErrorInfo,
    traits::{Constrainer, GenericConstrainer, TableRule},
};
use sql_traits::errors::LookupError;
use sql_traits::traits::IndexLike;
use sql_traits::traits::{DatabaseLike, TableLike};

/// Struct defining a constraint that enforces that table names are lowercase.
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `UniqueUniqueIndex` constraint.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = UniqueUniqueIndex::default().into();
///
/// let invalid_schema =
///     ParserDB::parse::<GenericDialect>("CREATE TABLE MyTable (id INT, UNIQUE (id), UNIQUE (id));").unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// let valid_schema = ParserDB::parse::<GenericDialect>("CREATE TABLE mytable (id INT, UNIQUE (id));").unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct UniqueUniqueIndex<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for UniqueUniqueIndex<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<UniqueUniqueIndex<DB>> for GenericConstrainer<DB> {
    fn from(constraint: UniqueUniqueIndex<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_table_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> TableRule for UniqueUniqueIndex<DB> {
    type Database = DB;

    fn validate_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), crate::error::Error<DB>> {
        let mut expressions = table
            .unique_indices(database)?
            .map(|unique_index| unique_index.expression(database))
            .collect::<Result<Vec<_>, LookupError>>()?;
        expressions.sort_unstable();
        for window in expressions.windows(2) {
            if window[0] == window[1] {
                let duplicate_expression = window[0];
                let error: RuleErrorInfo = RuleErrorInfo::builder()
                    .rule("UniqueUniqueIndex")
                    .unwrap()
                    .object(table.table_name().to_owned())
                    .unwrap()
                    .message(format!(
                        "Table '{}' has non-unique unique index on columns: {}",
                        table.table_name(),
                        duplicate_expression
                    ))
                    .unwrap()
                    .resolution("Ensure all unique index in the table are unique".to_string())
                    .unwrap()
                    .try_into()
                    .unwrap();
                return Err(crate::error::Error::Table(
                    Box::new(table.clone()),
                    error.into(),
                ));
            }
        }
        Ok(())
    }
}
