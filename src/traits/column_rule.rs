//! Submodule defining the `ColumnRule` trait, which defines a rule applied to
//! each column of a schema.

use sql_traits::traits::DatabaseLike;

use crate::{error::Interruption, lint::Rule, traits::Reporter};

/// Trait for types that define a column rule object.
pub trait ColumnRule: Rule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Reports every violation of this rule on the given column.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Lookup`] when a schema object the rule reads
    /// cannot be resolved, and [`Interruption::Halted`] when the reporter
    /// needs no further violations.
    fn check_column(
        &self,
        database: &Self::Database,
        column: &<Self::Database as DatabaseLike>::Column,
        reporter: &mut Reporter<'_, Self::Database>,
    ) -> Result<(), Interruption>;
}
