//! Submodule defining the `ForeignKeyRule` trait, which defines a
//! rule which applies to an object that implements the `ForeignKeyLike`
//! trait.

use sql_traits::traits::DatabaseLike;

use crate::error::Error;

/// Trait for types that define a foreign key rule object.
pub trait ForeignKeyRule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Validates that the given foreign key satisfies the rule.
    ///
    /// # Errors
    ///
    /// Returns an error if the foreign key violates this rule, or
    /// [`Error::Lookup`] if a schema object the rule reads cannot be resolved.
    fn validate_foreign_key(
        &self,
        database: &Self::Database,
        foreign_key: &<Self::Database as DatabaseLike>::ForeignKey,
    ) -> Result<(), Error<Self::Database>>;
}
