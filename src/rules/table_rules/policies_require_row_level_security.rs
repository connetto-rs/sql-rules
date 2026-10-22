//! Submodule providing the `PoliciesRequireRowLevelSecurity` constraint.

use sql_traits::traits::{DatabaseLike, TableLike};

use crate::{
    error::RuleErrorInfo,
    traits::{Constrainer, GenericConstrainer, TableRule},
};

/// Struct defining a constraint that enforces that if a table has policies, it must have RLS enabled.
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `PoliciesRequireRowLevelSecurity` constraint.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = PoliciesRequireRowLevelSecurity::default().into();
///
/// // Invalid: Has policy but RLS not enabled
/// let invalid_schema = ParserDB::parse::<GenericDialect>("
///     CREATE TABLE my_table (id INT);
///     CREATE POLICY p ON my_table USING (true);
/// ").unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// // Valid: Has policy and RLS enabled
/// let valid_schema = ParserDB::parse::<GenericDialect>("
///     CREATE TABLE my_table (id INT);
///     ALTER TABLE my_table ENABLE ROW LEVEL SECURITY;
///     CREATE POLICY p ON my_table USING (true);
/// ").unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct PoliciesRequireRowLevelSecurity<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for PoliciesRequireRowLevelSecurity<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<PoliciesRequireRowLevelSecurity<DB>>
    for GenericConstrainer<DB>
{
    fn from(constraint: PoliciesRequireRowLevelSecurity<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_table_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> TableRule for PoliciesRequireRowLevelSecurity<DB> {
    type Database = DB;

    fn validate_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), crate::error::Error<DB>> {
        let has_policies = table.policies(database)?.next().is_some();
        let is_rls_enabled = table.has_row_level_security(database)?;

        if has_policies && !is_rls_enabled {
            let error: RuleErrorInfo = RuleErrorInfo::builder()
                .rule("PoliciesRequireRowLevelSecurity")
                .unwrap()
                .object(table.table_name().to_owned())
                .unwrap()
                .message(format!(
                    "Table '{}' has policies but RLS is not enabled",
                    table.table_name()
                ))
                .unwrap()
                .resolution("Enable Row Level Security on the table".to_string())
                .unwrap()
                .try_into()
                .unwrap();
            Err(crate::error::Error::Table(
                Box::new(table.clone()),
                error.into(),
            ))
        } else {
            Ok(())
        }
    }
}
