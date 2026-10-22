//! Submodule providing the `PastTimeColumnRule` rule.

use crate::{
    error::RuleErrorInfo,
    traits::{ColumnRule, Constrainer, GenericConstrainer},
};
use sql_traits::traits::{CheckConstraintLike, ColumnLike, DatabaseLike, TableLike};

/// Struct defining a rule that enforces that time-related columns must have a check constraint ensuring they are in the past.
///
/// This rule checks if a column name ends with `_at` (e.g., `created_at`, `updated_at`).
/// If so, it requires a check constraint involving `NOW()` or `CURRENT_TIMESTAMP` and a less-than operator.
///
/// # Example
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = PastTimeColumnRule::default().into();
///
/// // Invalid: created_at without constraint
/// let invalid_schema = ParserDB::parse::<GenericDialect>("CREATE TABLE users (created_at TIMESTAMP);").unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// // Valid: created_at with constraint
/// let valid_schema = ParserDB::parse::<GenericDialect>("CREATE TABLE users (created_at TIMESTAMP CHECK (created_at <= NOW()));").unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct PastTimeColumnRule<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for PastTimeColumnRule<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<PastTimeColumnRule<DB>> for GenericConstrainer<DB> {
    fn from(rule: PastTimeColumnRule<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_column_rule(Box::new(rule));
        constrainer
    }
}

impl<DB: DatabaseLike> ColumnRule for PastTimeColumnRule<DB> {
    type Database = DB;

    fn validate_column(
        &self,
        database: &Self::Database,
        column: &<Self::Database as DatabaseLike>::Column,
    ) -> Result<(), crate::error::Error<DB>> {
        let column_name = column.column_name();

        // Identify time-related columns by suffix "_at"
        if !column_name.ends_with("_at") {
            return Ok(());
        }

        // Exclude common future or interval boundary columns
        let future_or_ambiguous = [
            "expires_at",
            "due_at",
            "starts_at",
            "ends_at",
            "scheduled_at",
        ];
        if future_or_ambiguous.contains(&column_name) {
            return Ok(());
        }

        let has_past_check = column.check_constraints(database)?.any(|cc| {
            let expr = cc.expression(database).to_string().to_lowercase();

            // Check for "now()" or "current_timestamp"
            let mentions_now = expr.contains("now()") || expr.contains("current_timestamp");

            // Check for less than operator
            let has_lt = expr.contains('<');

            mentions_now && has_lt
        });

        if !has_past_check {
            let table_name = column.table(database).table_name();
            let error: RuleErrorInfo = RuleErrorInfo::builder()
                .rule("PastTimeColumnRule")
                .unwrap()
                .object(format!("{table_name}.{column_name}"))
                .unwrap()
                .message(format!(
                    "Time-related column '{table_name}.{column_name}' must have a check constraint ensuring it is in the past."
                ))
                .unwrap()
                .resolution(format!(
                    "Add a check constraint like `CHECK ({column_name} <= NOW())`."
                ))
                .unwrap()
                .try_into()
                .unwrap();
            return Err(crate::error::Error::Column(
                Box::new(column.clone()),
                error.into(),
            ));
        }

        Ok(())
    }
}
