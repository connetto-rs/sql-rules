//! Submodule providing the `UniqueCheckConstraint` constraint, which enforces
//! that check constraints are unique per table, meaning that no two check
//! constraints have the same clause in a single table.

use sql_traits::traits::{CheckConstraintLike, DatabaseLike, TableLike};

use crate::{
    error::RuleErrorInfo,
    traits::{Constrainer, GenericConstrainer, TableRule},
};

/// Struct defining a constraint that enforces that table names are lowercase.
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `UniqueCheckRule` rule.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = UniqueCheckRule::default().into();
///
/// let invalid_schema =
///     ParserDB::parse::<GenericDialect>("CREATE TABLE MyTable (id INT, CHECK (id > 0), CHECK (id > 0));")
///         .unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// let valid_schema =
///     ParserDB::parse::<GenericDialect>("CREATE TABLE mytable (id INT, CHECK (id > 0));").unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct UniqueCheckRule<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for UniqueCheckRule<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<UniqueCheckRule<DB>> for GenericConstrainer<DB> {
    fn from(constraint: UniqueCheckRule<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_table_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> TableRule for UniqueCheckRule<DB> {
    type Database = DB;

    fn validate_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), crate::error::Error<DB>> {
        let mut constraints = table.check_constraints(database)?.collect::<Vec<_>>();
        constraints.sort_unstable_by_key(|c| c.expression(database));
        for window in constraints.windows(2) {
            if window[0].expression(database) == window[1].expression(database) {
                let error: RuleErrorInfo = RuleErrorInfo::builder()
                    .rule("UniqueCheckConstraint")
                    .unwrap()
                    .object(table.table_name().to_owned())
                    .unwrap()
                    .message(format!(
                        "Table '{}' has non-unique check constraints",
                        table.table_name()
                    ))
                    .unwrap()
                    .resolution("Ensure all check constraints in the table are unique".to_string())
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
