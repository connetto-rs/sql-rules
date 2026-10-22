//! Submodule providing the `NoTautologicalCheckRule` rule, which
//! enforces that tables do not have tautological (always true) check
//! constraints.

use sql_traits::traits::{CheckConstraintLike, DatabaseLike, TableLike};

use crate::{
    error::RuleErrorInfo,
    traits::{Constrainer, GenericConstrainer, TableRule},
};

/// Struct defining a constraint that enforces that tables do not have
/// tautological check constraints.
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `NoTautologicalCheckRule` rule.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = NoTautologicalCheckRule::default().into();
///
/// // Invalid: has tautological check constraint CHECK (true)
/// let invalid_schema = ParserDB::parse::<GenericDialect>(
///     "CREATE TABLE my_table (
///         id INT PRIMARY KEY,
///         age INT CHECK (true)
///     );",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// // Invalid: has tautological check constraint CHECK (1 = 1)
/// let invalid_schema2 = ParserDB::parse::<GenericDialect>(
///     "CREATE TABLE my_table (
///         id INT PRIMARY KEY,
///         age INT CHECK (1 = 1)
///     );",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema2).is_err());
///
/// // Valid: has meaningful check constraint
/// let valid_schema = ParserDB::parse::<GenericDialect>(
///     "CREATE TABLE my_table (
///         id INT PRIMARY KEY,
///         age INT CHECK (age > 0)
///     );",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct NoTautologicalCheckRule<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for NoTautologicalCheckRule<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<NoTautologicalCheckRule<DB>> for GenericConstrainer<DB> {
    fn from(constraint: NoTautologicalCheckRule<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_table_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> TableRule for NoTautologicalCheckRule<DB> {
    type Database = DB;

    fn validate_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), crate::error::Error<DB>> {
        // Check if any check constraint is tautological
        for check_constraint in table.check_constraints(database)? {
            if check_constraint.is_tautology(database)? {
                let table_name = table.table_name();
                let tautological_constraint = check_constraint.expression(database).to_string();

                let error: RuleErrorInfo = RuleErrorInfo::builder()
                    .rule("NoTautologicalCheckRule")
                    .unwrap()
                    .object(table_name.to_owned())
                    .unwrap()
                    .message(format!(
                        "Table '{table_name}' has a tautological check constraint: CHECK ({tautological_constraint})"
                    ))
                    .unwrap()
                    .resolution(format!(
                        "Remove the tautological check constraint 'CHECK ({tautological_constraint})' from table '{table_name}'"
                    ))
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
