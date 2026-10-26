//! Submodule defining the `TableRule` trait, which defines a rule applied to
//! each table of a schema.

use sql_traits::traits::DatabaseLike;

use crate::{error::Interruption, lint::Rule, traits::Reporter};

/// Trait for types that define a table rule object.
pub trait TableRule: Rule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Reports every violation of this rule on the given table.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Lookup`] when a schema object the rule reads
    /// cannot be resolved, and [`Interruption::Halted`] when the reporter
    /// needs no further violations.
    fn check_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
        reporter: &mut Reporter<'_, Self::Database>,
    ) -> Result<(), Interruption>;
}
