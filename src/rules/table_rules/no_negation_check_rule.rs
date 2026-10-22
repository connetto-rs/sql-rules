//! Submodule providing the `NoNegationCheckRule` rule, which
//! enforces that tables do not have negation (always false) check
//! constraints.

use sql_traits::traits::{CheckConstraintLike, DatabaseLike, TableLike};

use crate::{
    error::RuleErrorInfo,
    traits::{Constrainer, GenericConstrainer, TableRule},
};

/// Struct defining a constraint that enforces that tables do not have
/// negation check constraints.
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `NoNegationCheckRule` rule.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = NoNegationCheckRule::default().into();
///
/// // Invalid: has negation check constraint CHECK (false)
/// let invalid_schema = ParserDB::parse::<GenericDialect>(
///     "CREATE TABLE my_table (
///         id INT PRIMARY KEY,
///         age INT CHECK (false)
///     );",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// // Invalid: has negation check constraint CHECK (1 = 0)
/// let invalid_schema2 = ParserDB::parse::<GenericDialect>(
///     "CREATE TABLE my_table (
///         id INT PRIMARY KEY,
///         age INT CHECK (1 = 0)
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
pub struct NoNegationCheckRule<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for NoNegationCheckRule<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<NoNegationCheckRule<DB>> for GenericConstrainer<DB> {
    fn from(constraint: NoNegationCheckRule<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_table_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> TableRule for NoNegationCheckRule<DB> {
    type Database = DB;

    fn validate_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), crate::error::Error<DB>> {
        let mut negation_constraint = None;
        for cc in table.check_constraints(database)? {
            if cc.is_negation(database)? {
                negation_constraint = Some(cc);
                break;
            }
        }

        if let Some(negation_constraint) = negation_constraint {
            let table_name = table.table_name();
            let negation_constraint = negation_constraint.expression(database).to_string();

            let error: RuleErrorInfo = RuleErrorInfo::builder()
                .rule("NoNegationCheckRule")
                .unwrap()
                .object(table_name.to_owned())
                .unwrap()
                .message(format!(
                    "Table '{table_name}' has a negation check constraint: CHECK ({negation_constraint})"
                ))
                .unwrap()
                .resolution("Remove the negation check constraint.".to_string())
                .unwrap()
                .try_into()
                .unwrap();
            return Err(crate::error::Error::Table(
                Box::new(table.clone()),
                error.into(),
            ));
        }
        Ok(())
    }
}
