//! Submodule providing the `NoRustKeywordForeignKeyName` constraint, which
//! enforces that foreign key names are not Rust keywords.

use sql_traits::traits::{DatabaseLike, ForeignKeyLike};

use crate::{
    error::RuleErrorInfo,
    rules::rust_keywords::is_rust_keyword,
    traits::{Constrainer, ForeignKeyRule, GenericConstrainer},
};

/// Struct defining a constraint that enforces that foreign key names are not
/// Rust keywords.
///
/// # Example
///
/// Here follows an example of validating an invalid SQL statement with the
/// `NoRustKeywordForeignKeyName` constraint.
///
/// ```rust
/// use sql_rules::prelude::*;
/// use sqlparser::dialect::GenericDialect;
///
/// let constrainer: GenericConstrainer<ParserDB> = NoRustKeywordForeignKeyName::default().into();
///
/// let invalid_schema = ParserDB::parse::<GenericDialect>("CREATE TABLE other_table (id INT PRIMARY KEY); CREATE TABLE mytable (id INT, CONSTRAINT struct FOREIGN KEY (id) REFERENCES other_table (id));").unwrap();
/// assert!(constrainer.validate_schema(&invalid_schema).is_err());
///
/// let valid_schema = ParserDB::parse::<GenericDialect>("CREATE TABLE other_table (id INT PRIMARY KEY); CREATE TABLE mytable (id INT, CONSTRAINT my_struct FOREIGN KEY (id) REFERENCES other_table (id));").unwrap();
/// assert!(constrainer.validate_schema(&valid_schema).is_ok());
/// ```
pub struct NoRustKeywordForeignKeyName<DB>(std::marker::PhantomData<DB>);

impl<DB> Default for NoRustKeywordForeignKeyName<DB> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB: DatabaseLike + 'static> From<NoRustKeywordForeignKeyName<DB>> for GenericConstrainer<DB> {
    fn from(constraint: NoRustKeywordForeignKeyName<DB>) -> Self {
        let mut constrainer = GenericConstrainer::default();
        constrainer.register_foreign_key_rule(Box::new(constraint));
        constrainer
    }
}

impl<DB: DatabaseLike> ForeignKeyRule for NoRustKeywordForeignKeyName<DB> {
    type Database = DB;

    fn validate_foreign_key(
        &self,
        _database: &Self::Database,
        foreign_key: &<Self::Database as DatabaseLike>::ForeignKey,
    ) -> Result<(), crate::error::Error<DB>> {
        if let Some(name) = foreign_key.foreign_key_name()
            && is_rust_keyword(name)
        {
            let error: RuleErrorInfo = RuleErrorInfo::builder()
                .rule("NoRustKeywordForeignKeyName")
                .unwrap()
                .object(name.to_owned())
                .unwrap()
                .message(format!("Foreign key name '{name}' is a Rust keyword."))
                .unwrap()
                .resolution(format!(
                    "Rename the foreign key '{name}' to something that is not a Rust keyword."
                ))
                .unwrap()
                .try_into()
                .unwrap();
            return Err(crate::error::Error::ForeignKey(
                Box::new(foreign_key.clone()),
                error.into(),
            ));
        }
        Ok(())
    }
}
