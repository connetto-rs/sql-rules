//! Submodule defining the `Constrainer` trait, which defines an object that
//! executes registered rules while visiting a schema.

use crate::{
    error::Error,
    traits::{ColumnRule, ForeignKeyRule, TableRule},
};

pub mod generic_constrainer;
pub use generic_constrainer::GenericConstrainer;
pub mod default_constrainer;
pub use default_constrainer::DefaultConstrainer;
use sql_traits::traits::{DatabaseLike, TableLike};

/// Trait for types that define a constrainer object.
pub trait Constrainer: Default {
    /// Associated database type for the constrainer.
    type Database: DatabaseLike;

    /// Registers a table rule to be applied to a table.
    fn register_table_rule(&mut self, rule: Box<dyn TableRule<Database = Self::Database>>);

    /// Registers a column rule to be applied to a column.
    fn register_column_rule(&mut self, rule: Box<dyn ColumnRule<Database = Self::Database>>);

    /// Registers a foreign key rule to be applied to a foreign key.
    fn register_foreign_key_rule(
        &mut self,
        rule: Box<dyn ForeignKeyRule<Database = Self::Database>>,
    );

    /// Returns an iterator over all registered table rules.
    fn table_rules(&self) -> impl Iterator<Item = &dyn TableRule<Database = Self::Database>>;

    /// Returns an iterator over all registered column rules.
    fn column_rules(&self) -> impl Iterator<Item = &dyn ColumnRule<Database = Self::Database>>;

    /// Returns an iterator over all registered foreign key rules.
    fn foreign_key_rules(
        &self,
    ) -> impl Iterator<Item = &dyn ForeignKeyRule<Database = Self::Database>>;

    /// Encounters a table and applies all registered table rules to it.
    ///
    /// # Errors
    ///
    /// Returns an error if any table rule is violated, or [`Error::Lookup`] if
    /// a schema object a rule reads cannot be resolved.
    fn encounter_table(
        &self,
        database: &Self::Database,
        table: &<Self::Database as DatabaseLike>::Table,
    ) -> Result<(), Error<Self::Database>> {
        self.table_rules()
            .try_for_each(|constraint| constraint.validate_table(database, table))
    }

    /// Encounters a column and applies all registered column rules to it.
    ///
    /// # Errors
    ///
    /// Returns an error if any column rule is violated, or [`Error::Lookup`]
    /// if a schema object a rule reads cannot be resolved.
    fn encounter_column(
        &self,
        database: &Self::Database,
        column: &<Self::Database as DatabaseLike>::Column,
    ) -> Result<(), Error<Self::Database>> {
        self.column_rules()
            .try_for_each(|constraint| constraint.validate_column(database, column))
    }

    /// Encounters a foreign key and applies all registered foreign key
    /// rules to it.
    ///
    /// # Errors
    ///
    /// Returns an error if any foreign key rule is violated, or
    /// [`Error::Lookup`] if a schema object a rule reads cannot be resolved.
    fn encounter_foreign_key(
        &self,
        database: &Self::Database,
        foreign_key: &<Self::Database as DatabaseLike>::ForeignKey,
    ) -> Result<(), Error<Self::Database>> {
        self.foreign_key_rules()
            .try_for_each(|constraint| constraint.validate_foreign_key(database, foreign_key))
    }

    /// Validates the provided schema by applying all registered rules to
    /// its DB entities.
    ///
    /// # Errors
    ///
    /// Returns an error if any rule is violated, or [`Error::Lookup`] if a
    /// table's columns or foreign keys, or a schema object a rule reads,
    /// cannot be resolved.
    fn validate_schema(&self, database: &Self::Database) -> Result<(), Error<Self::Database>> {
        for table in database.tables() {
            self.encounter_table(database, table)?;
            for column in table.columns(database)? {
                self.encounter_column(database, column)?;
            }
            for foreign_key in table.foreign_keys(database)? {
                self.encounter_foreign_key(database, foreign_key)?;
            }
        }
        Ok(())
    }
}
