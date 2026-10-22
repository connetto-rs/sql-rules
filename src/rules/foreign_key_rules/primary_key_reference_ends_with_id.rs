//! Submodule providing the `PrimaryKeyReferenceEndsWithId` constraint, which
//! enforces that foreign key host columns that reference primary key columns
//! (directly or indirectly) must have names ending with "id".

use sql_traits::{
    errors::LookupError,
    traits::{ColumnLike, DatabaseLike, ForeignKeyLike, TableLike},
};

use crate::{
    error::RuleErrorInfo,
    traits::{Constrainer, ForeignKeyRule, GenericConstrainer},
};

/// Struct defining a constraint that enforces that foreign key host columns
/// that reference primary key columns (directly or indirectly) must have names
/// ending with "id".
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `PrimaryKeyReferenceEndsWithId` constraint.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = PrimaryKeyReferenceEndsWithId::default().into();
///
/// let invalid_schema = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE parent (id INT PRIMARY KEY);
/// CREATE TABLE child (parent_key INT, FOREIGN KEY (parent_key) REFERENCES parent (id));
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// let valid_schema_direct = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE parent (id INT PRIMARY KEY);
/// CREATE TABLE child (parent_id INT, FOREIGN KEY (parent_id) REFERENCES parent (id));
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_schema_direct).is_ok());
///
/// let valid_schema_indirect = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE grandparent (id INT PRIMARY KEY);
/// CREATE TABLE parent (gp_id INT UNIQUE, FOREIGN KEY (gp_id) REFERENCES grandparent (id));
/// CREATE TABLE child (parent_id INT, FOREIGN KEY (parent_id) REFERENCES parent (gp_id));
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_schema_indirect).is_ok());
///
/// let valid_schema_non_pk = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE parent (id INT PRIMARY KEY, code TEXT UNIQUE);
/// CREATE TABLE child (parent_code TEXT, FOREIGN KEY (parent_code) REFERENCES parent (code));
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_schema_non_pk).is_ok());
///
/// // `parent.owner_id` references a primary key, but `child` references `parent.code`
/// let valid_schema_other_fk = ParserDB::parse::<GenericDialect>(
///     "
/// CREATE TABLE owner (id INT PRIMARY KEY);
/// CREATE TABLE parent (id INT PRIMARY KEY, code INT UNIQUE, owner_id INT REFERENCES owner (id));
/// CREATE TABLE child (parent_code INT, FOREIGN KEY (parent_code) REFERENCES parent (code));
/// ",
/// )
/// .unwrap();
/// assert!(constrainer.validate_schema(&valid_schema_other_fk).is_ok());
/// ```
pub struct PrimaryKeyReferenceEndsWithId<C>(std::marker::PhantomData<C>);

impl<C> Default for PrimaryKeyReferenceEndsWithId<C> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<PrimaryKeyReferenceEndsWithId<DB>>
    for GenericConstrainer<DB>
{
    fn from(constraint: PrimaryKeyReferenceEndsWithId<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_foreign_key_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> PrimaryKeyReferenceEndsWithId<DB> {
    /// Returns whether `column` of `table` is the host column of a foreign key
    /// whose referenced columns include a primary key column.
    fn references_primary_key_through_foreign_key(
        database: &DB,
        table: &DB::Table,
        column: &DB::Column,
    ) -> Result<bool, LookupError> {
        for fk in table.foreign_keys(database)? {
            if !fk
                .host_columns(database)?
                .any(|fk_host_col| fk_host_col.column_name() == column.column_name())
            {
                continue;
            }
            for fk_ref_col in fk.referenced_columns(database)? {
                let fk_ref_table = fk.referenced_table(database)?;
                if fk_ref_table.is_primary_key_column(database, fk_ref_col)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl<DB: DatabaseLike> ForeignKeyRule for PrimaryKeyReferenceEndsWithId<DB> {
    type Database = DB;

    fn validate_foreign_key(
        &self,
        database: &Self::Database,
        foreign_key: &<Self::Database as DatabaseLike>::ForeignKey,
    ) -> Result<(), crate::prelude::Error<DB>> {
        let host_table = foreign_key.host_table(database);
        let referenced_table = foreign_key.referenced_table(database)?;

        for (host_column, referenced_column) in foreign_key
            .host_columns(database)?
            .zip(foreign_key.referenced_columns(database)?)
        {
            // Check if the referenced column is a primary key or references a primary key
            let references_primary_key = referenced_table
                .is_primary_key_column(database, referenced_column)?
                || Self::references_primary_key_through_foreign_key(
                    database,
                    referenced_table,
                    referenced_column,
                )?;

            if references_primary_key && !host_column.column_name().ends_with("id") {
                let error: RuleErrorInfo = RuleErrorInfo::builder()
                    .rule("PrimaryKeyReferenceEndsWithId")
                    .unwrap()
                    .object(
                        foreign_key
                            .foreign_key_name()
                            .unwrap_or("Unnamed foreign key")
                            .to_owned(),
                    )
                    .unwrap()
                    .message(format!(
                        "Foreign key host column '{}.{}' references a primary key column '{}.{}' but does not end with 'id'",
                        host_table.table_name(),
                        host_column.column_name(),
                        referenced_table.table_name(),
                        referenced_column.column_name(),
                    ))
                    .unwrap()
                    .resolution(format!(
                        "Rename column '{}.{}' to end with 'id' (e.g., '{}_id')",
                        host_table.table_name(),
                        host_column.column_name(),
                        host_column.column_name(),
                    ))
                    .unwrap()
                    .try_into()
                    .unwrap();
                return Err(crate::error::Error::ForeignKey(
                    Box::new(foreign_key.clone()),
                    error.into(),
                ));
            }
        }

        Ok(())
    }
}
