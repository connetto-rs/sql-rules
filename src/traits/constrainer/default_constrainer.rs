//! Submodule defining a default constrainer with all available constraints.

use sql_traits::traits::DatabaseLike;

use crate::{
    prelude::{
        ExtensionForeignKeyOnDeleteCascade, NoNegationCheckRule, NoRustKeywordColumnName,
        NoRustKeywordForeignKeyName, NoRustKeywordTableName, NoTautologicalCheckRule,
        PrimaryKeyReferenceEndsWithId,
    },
    rules::{
        CompatibleForeignKey, HasPrimaryKey, LowercaseColumnName, LowercaseForeignKeyName,
        LowercaseTableName, NoForbiddenColumnInExtension, NoSurrogatePrimaryKeyInExtension,
        NonCompositePrimaryKeyNamedId, NonRedundantExtensionDag, PastTimeColumnRule,
        PluralTableName, PoliciesRequireRowLevelSecurity, SingularColumnName, SnakeCaseColumnName,
        SnakeCaseTableName, TextualColumnRule, UniqueCheckRule, UniqueColumnNamesInExtensionGraph,
        UniqueForeignKey, UniqueUniqueIndex,
    },
    traits::Constrainer,
};

/// A constrainer that comes pre-configured with all available constraints.
///
/// This struct provides a `Default` implementation that registers all available
/// table, column, and foreign key constraints. This is useful for ensuring
/// comprehensive validation of database schemas.
///
/// # Available Constraints
///
/// ## Table Constraints
/// - [`HasPrimaryKey`]: Ensures all tables have a primary key
/// - [`LowercaseTableName`]: Ensures table names are lowercase
/// - [`SnakeCaseTableName`]: Ensures table names follow `snake_case` convention
/// - [`PluralTableName`]: Ensures table names are plural
/// - [`PoliciesRequireRowLevelSecurity`]: Ensures tables with policies have RLS enabled
/// - [`NoForbiddenColumnInExtension`]: Prevents forbidden columns in extended
///   tables
/// - [`NonRedundantExtensionDag`]: Ensures no redundant edges in extension
///   hierarchy
/// - [`UniqueCheckRule`]: Ensures check constraint names are unique
/// - [`UniqueColumnNamesInExtensionGraph`]: Ensures column names are unique
///   across extension graphs
/// - [`UniqueForeignKey`]: Ensures foreign key signatures are unique
/// - [`UniqueUniqueIndex`]: Ensures unique index names are unique
///
/// ## Column Constraints
/// - [`LowercaseColumnName`]: Ensures column names are lowercase
/// - [`NonCompositePrimaryKeyNamedId`]: Ensures non-composite primary keys are
///   named "id"
/// - [`NoSurrogatePrimaryKeyInExtension`]: Ensures extension-table primary
///   keys are not surrogate
/// - [`SnakeCaseColumnName`]: Ensures column names follow `snake_case`
///   convention
/// - [`SingularColumnName`]: Ensures column names are singular
/// - [`TextualColumnRule`]: Ensures textual columns have content and length checks
/// - [`PastTimeColumnRule`]: Ensures time-related columns are in the past
///
/// ## Foreign Key Constraints
/// - [`CompatibleForeignKey`]: Ensures foreign key columns are type-compatible
/// - [`LowercaseForeignKeyName`]: Ensures foreign key names are lowercase
///
/// # Example
///
/// ```
/// use sql_rules::prelude::*;
///
/// let constrainer = DefaultConstrainer::<ParserDB>::default();
/// // All constraints are now registered and ready to use
/// ```
pub struct DefaultConstrainer<DB: DatabaseLike> {
    /// The underlying generic constrainer holding all constraints.
    constrainer: super::generic_constrainer::GenericConstrainer<DB>,
}

impl<DB: DatabaseLike + 'static> Default for DefaultConstrainer<DB>
where
    DB::Column: 'static,
{
    fn default() -> Self {
        let mut constrainer = super::generic_constrainer::GenericConstrainer::default();

        // Register all table constraints
        constrainer.register_table_rule(Box::new(HasPrimaryKey::default()));
        constrainer.register_table_rule(Box::new(LowercaseTableName::default()));
        constrainer.register_table_rule(Box::new(SnakeCaseTableName::default()));
        constrainer.register_table_rule(Box::new(PluralTableName::default()));
        constrainer.register_table_rule(Box::new(PoliciesRequireRowLevelSecurity::default()));
        constrainer.register_table_rule(Box::new(NoRustKeywordTableName::default()));
        constrainer.register_table_rule(Box::new(NoTautologicalCheckRule::default()));
        constrainer.register_table_rule(Box::new(NoNegationCheckRule::default()));
        constrainer.register_table_rule(Box::new(NoForbiddenColumnInExtension::new(
            "most_concrete_table",
        )));
        constrainer.register_table_rule(Box::new(NonRedundantExtensionDag::default()));
        constrainer.register_table_rule(Box::new(UniqueCheckRule::default()));
        constrainer.register_table_rule(Box::new(UniqueColumnNamesInExtensionGraph::default()));
        constrainer.register_table_rule(Box::new(UniqueForeignKey::default()));
        constrainer.register_table_rule(Box::new(UniqueUniqueIndex::default()));

        // Register all column constraints
        constrainer.register_column_rule(Box::new(LowercaseColumnName::default()));
        constrainer.register_column_rule(Box::new(NonCompositePrimaryKeyNamedId::default()));
        constrainer.register_column_rule(Box::new(NoSurrogatePrimaryKeyInExtension::default()));
        constrainer.register_column_rule(Box::new(SnakeCaseColumnName::default()));
        constrainer.register_column_rule(Box::new(SingularColumnName::default()));
        constrainer.register_column_rule(Box::new(NoRustKeywordColumnName::default()));
        constrainer.register_column_rule(Box::new(TextualColumnRule::default()));
        constrainer.register_column_rule(Box::new(PastTimeColumnRule::default()));

        // Register all foreign key constraints
        constrainer.register_foreign_key_rule(Box::new(CompatibleForeignKey::default()));
        constrainer.register_foreign_key_rule(Box::new(LowercaseForeignKeyName::default()));
        constrainer.register_foreign_key_rule(Box::new(PrimaryKeyReferenceEndsWithId::default()));
        constrainer
            .register_foreign_key_rule(Box::new(ExtensionForeignKeyOnDeleteCascade::default()));
        constrainer.register_foreign_key_rule(Box::new(NoRustKeywordForeignKeyName::default()));

        Self { constrainer }
    }
}

impl<DB: DatabaseLike + 'static> Constrainer for DefaultConstrainer<DB>
where
    DB::Column: 'static,
{
    type Database = DB;

    fn table_rules(
        &self,
    ) -> impl Iterator<Item = &dyn crate::traits::TableRule<Database = Self::Database>> {
        self.constrainer.table_rules()
    }

    fn column_rules(
        &self,
    ) -> impl Iterator<Item = &dyn crate::traits::ColumnRule<Database = Self::Database>> {
        self.constrainer.column_rules()
    }

    fn foreign_key_rules(
        &self,
    ) -> impl Iterator<Item = &dyn crate::traits::ForeignKeyRule<Database = Self::Database>> {
        self.constrainer.foreign_key_rules()
    }

    fn register_table_rule(
        &mut self,
        rule: Box<dyn crate::traits::TableRule<Database = Self::Database>>,
    ) {
        self.constrainer.register_table_rule(rule);
    }

    fn register_column_rule(
        &mut self,
        rule: Box<dyn crate::traits::ColumnRule<Database = Self::Database>>,
    ) {
        self.constrainer.register_column_rule(rule);
    }

    fn register_foreign_key_rule(
        &mut self,
        rule: Box<dyn crate::traits::ForeignKeyRule<Database = Self::Database>>,
    ) {
        self.constrainer.register_foreign_key_rule(rule);
    }
}
