//! Violations and failures raised while applying rules.

use std::fmt::{self, Display};

use sql_traits::{errors::LookupError, traits::DatabaseLike};

use crate::lint::{Level, RuleMetadata};

/// Schema object a violation was raised on.
#[derive(Debug)]
pub enum Location<DB: DatabaseLike> {
    /// A table.
    Table(Box<DB::Table>),
    /// A column.
    Column(Box<DB::Column>),
    /// A foreign key.
    ForeignKey(Box<DB::ForeignKey>),
}

/// One occurrence of a rule violation.
#[derive(Debug)]
pub struct Violation<DB: DatabaseLike> {
    pub(crate) rule: &'static RuleMetadata,
    pub(crate) level: Level,
    pub(crate) location: Location<DB>,
    pub(crate) object: String,
    pub(crate) message: String,
    pub(crate) resolution: String,
}

impl<DB: DatabaseLike> Violation<DB> {
    /// Returns the metadata of the violated rule.
    #[must_use]
    pub fn rule(&self) -> &'static RuleMetadata {
        self.rule
    }

    /// Returns the level the violation was raised at.
    #[must_use]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns the schema object the violation was raised on.
    #[must_use]
    pub fn location(&self) -> &Location<DB> {
        &self.location
    }

    /// Returns the name of the offending schema object.
    #[must_use]
    pub fn object(&self) -> &str {
        &self.object
    }

    /// Returns what is wrong with this object.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the corrected shape.
    #[must_use]
    pub fn resolution(&self) -> &str {
        &self.resolution
    }
}

impl<DB: DatabaseLike> Display for Violation<DB> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} [{}::{}] {}: {}\n  why: {}\n  fix: {}",
            self.level,
            self.rule.group,
            self.rule.name,
            self.object,
            self.message,
            self.rule.rationale,
            self.resolution,
        )?;
        if let Some(source) = self.rule.source {
            write!(formatter, "\n  source: {source}")?;
        }
        Ok(())
    }
}

/// Failure raised while applying rules.
#[derive(Debug, thiserror::Error)]
pub enum Error<DB: DatabaseLike> {
    /// A rule was violated at `warn` or `deny`.
    #[error("{0}")]
    Violation(Box<Violation<DB>>),
    /// A rule could not resolve a schema object it depends on, such as the
    /// table a foreign key references.
    #[error("Schema lookup failed: {0}")]
    Lookup(#[from] LookupError),
}

/// Why a rule stopped before checking everything it covers.
#[derive(Debug)]
pub enum Interruption {
    /// A schema object the rule reads could not be resolved.
    Lookup(LookupError),
    /// The caller asked to stop after the last reported violation.
    Halted,
}

impl From<LookupError> for Interruption {
    fn from(error: LookupError) -> Self {
        Self::Lookup(error)
    }
}
