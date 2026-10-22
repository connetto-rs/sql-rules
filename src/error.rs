//! Submodule defining the error enumeration which may occur when applying
//! rules.

mod rule_error_info;
pub use rule_error_info::RuleErrorInfo;
use sql_traits::{errors::LookupError, traits::DatabaseLike};

use crate::traits::RuleFailureInformation;

#[derive(Debug, thiserror::Error)]
/// Enumeration of possible errors that may occur when applying rules.
pub enum Error<DB: DatabaseLike> {
    #[error("Table rule violated: {1}")]
    /// Error indicating that a table rule was violated.
    Table(Box<DB::Table>, Box<dyn RuleFailureInformation>),
    /// Unapplicable rule error.
    #[error("Unapplicable rule: {0}")]
    Unapplicable(String),
    #[error("Column rule violated: {1}")]
    /// Error indicating that a column rule was violated.
    Column(Box<DB::Column>, Box<dyn RuleFailureInformation>),
    #[error("Foreign key rule violated: {1}")]
    /// Error indicating that a foreign key rule was violated.
    ForeignKey(Box<DB::ForeignKey>, Box<dyn RuleFailureInformation>),
    #[error("Schema lookup failed: {0}")]
    /// Error indicating that a rule could not resolve a schema object it
    /// depends on, such as the table a foreign key references.
    Lookup(#[from] LookupError),
}
