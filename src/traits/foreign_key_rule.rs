//! Submodule defining the `ForeignKeyRule` trait, which defines a rule applied
//! to each foreign key of a schema.

use sql_traits::traits::DatabaseLike;

use crate::{error::Interruption, lint::Rule, traits::Reporter};

/// Trait for types that define a foreign key rule object.
pub trait ForeignKeyRule: Rule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Reports every violation of this rule on the given foreign key.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Lookup`] when a schema object the rule reads
    /// cannot be resolved, and [`Interruption::Halted`] when the reporter
    /// needs no further violations.
    fn check_foreign_key(
        &self,
        database: &Self::Database,
        foreign_key: &<Self::Database as DatabaseLike>::ForeignKey,
        reporter: &mut Reporter<'_, Self::Database>,
    ) -> Result<(), Interruption>;
}
