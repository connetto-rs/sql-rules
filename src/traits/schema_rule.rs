//! Submodule defining the `SchemaRule` trait, which defines a rule applied to
//! a schema as a whole.

use sql_traits::traits::DatabaseLike;

use crate::{error::Interruption, lint::Rule, traits::Reporter};

/// Trait for types that define a rule spanning several schema objects.
pub trait SchemaRule: Rule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Reports every violation of this rule in the given schema.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Lookup`] when a schema object the rule reads
    /// cannot be resolved, and [`Interruption::Halted`] when the reporter
    /// needs no further violations.
    fn check_schema(
        &self,
        database: &Self::Database,
        reporter: &mut Reporter<'_, Self::Database>,
    ) -> Result<(), Interruption>;
}
