//! Submodule defining the `ColumnRule` trait, which defines a rule
//! which applies to an object that implements the `ConstrainableColumn` trait.

use sql_traits::traits::DatabaseLike;

use crate::error::Error;

/// Trait for types that define a column rule object.
pub trait ColumnRule {
    /// The database type that this rule applies to.
    type Database: DatabaseLike;

    /// Validates that the given column satisfies the rule.
    ///
    /// # Arguments
    ///
    /// * `database` - A reference to the database instance to query additional
    ///   column information from.
    /// * `column` - The column to validate.
    ///
    /// # Errors
    ///
    /// Returns an error if the column violates this rule, or
    /// [`Error::Lookup`] if a schema object the rule reads cannot be resolved.
    fn validate_column(
        &self,
        database: &Self::Database,
        column: &<Self::Database as DatabaseLike>::Column,
    ) -> Result<(), Error<Self::Database>>;
}
