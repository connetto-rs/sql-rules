//! Submodule defining the `Reporter` through which rules raise violations.

use std::ops::ControlFlow;

use sql_traits::traits::{ColumnLike, DatabaseLike, ForeignKeyLike, TableLike};

use crate::{
    error::{Error, Interruption, Location, Violation},
    lint::{Level, RuleMetadata},
};

/// Receives the violations of one rule and stamps them with its identity and
/// effective level.
pub struct Reporter<'run, DB: DatabaseLike> {
    database: &'run DB,
    rule: &'static RuleMetadata,
    level: Level,
    sink: &'run mut dyn FnMut(Error<DB>) -> ControlFlow<()>,
}

impl<'run, DB: DatabaseLike> Reporter<'run, DB> {
    pub(crate) fn new(
        database: &'run DB,
        rule: &'static RuleMetadata,
        level: Level,
        sink: &'run mut dyn FnMut(Error<DB>) -> ControlFlow<()>,
    ) -> Self {
        Self {
            database,
            rule,
            level,
            sink,
        }
    }

    /// Reports a violation on a table.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Halted`] when the caller needs no further
    /// violations.
    pub fn table(
        &mut self,
        table: &DB::Table,
        message: String,
        resolution: String,
    ) -> Result<(), Interruption> {
        let object = table.table_name().to_owned();
        self.emit(
            Location::Table(Box::new(table.clone())),
            object,
            message,
            resolution,
        )
    }

    /// Reports a violation on a column.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Halted`] when the caller needs no further
    /// violations.
    pub fn column(
        &mut self,
        column: &DB::Column,
        message: String,
        resolution: String,
    ) -> Result<(), Interruption> {
        let object = format!(
            "{}.{}",
            column.table(self.database).table_name(),
            column.column_name()
        );
        self.emit(
            Location::Column(Box::new(column.clone())),
            object,
            message,
            resolution,
        )
    }

    /// Reports a violation on a foreign key.
    ///
    /// # Errors
    ///
    /// Returns [`Interruption::Halted`] when the caller needs no further
    /// violations.
    pub fn foreign_key(
        &mut self,
        foreign_key: &DB::ForeignKey,
        message: String,
        resolution: String,
    ) -> Result<(), Interruption> {
        let host = foreign_key.host_table(self.database).table_name();
        let object = match foreign_key.foreign_key_name() {
            Some(name) => format!("{host}.{name}"),
            None => format!("{host} (unnamed foreign key)"),
        };
        self.emit(
            Location::ForeignKey(Box::new(foreign_key.clone())),
            object,
            message,
            resolution,
        )
    }

    fn emit(
        &mut self,
        location: Location<DB>,
        object: String,
        message: String,
        resolution: String,
    ) -> Result<(), Interruption> {
        let violation = Violation {
            rule: self.rule,
            level: self.level,
            location,
            object,
            message,
            resolution,
        };
        match (self.sink)(Error::Violation(Box::new(violation))) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(()) => Err(Interruption::Halted),
        }
    }
}
