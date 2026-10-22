//! Submodule defining the `TableRule` trait, which defines a rule
//! which applies to an object that implements the `ConstrainableTable` trait.

use sql_traits::traits::DatabaseLike;

use crate::error::Error;

/// Trait for types that define a table rule object.
pub trait TableRule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Validates that the given table satisfies the rule.
    ///
    /// # Errors
    ///
    /// Returns an error if the table violates this rule, or
    /// [`Error::Lookup`] if a schema object the rule reads cannot be resolved.
    fn validate_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), Error<Self::Database>>;
}
