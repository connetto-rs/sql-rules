//! Submodule defining a generic constrainer for SQL rules.

use std::ops::ControlFlow;

use sql_traits::traits::{DatabaseLike, TableLike};

use crate::{
    error::{Error, Interruption},
    lint::{Level, LintGroup, Rule, RuleMetadata},
    traits::{ColumnRule, Constrainer, ForeignKeyRule, Reporter, TableRule},
};

/// A registered rule with its configured level.
struct Registered<R: ?Sized> {
    rule: Box<R>,
    floor: Level,
    level: Level,
}

/// Level overrides set on a constrainer.
#[derive(Default)]
struct Levels {
    rules: Vec<(String, Level)>,
    groups: [Option<Level>; LintGroup::ALL.len()],
}

impl Levels {
    fn resolve(&self, metadata: &RuleMetadata, floor: Level) -> Level {
        self.rules
            .iter()
            .find_map(|(name, level)| (name == metadata.name).then_some(*level))
            .or(self.groups[metadata.group.index()])
            .unwrap_or_else(|| metadata.group.default_level().max(floor))
    }

    fn refresh<R: Rule + ?Sized>(&self, registered: &mut [Registered<R>]) {
        for entry in registered {
            entry.level = self.resolve(entry.rule.metadata(), entry.floor);
        }
    }
}

/// A constrainer holding rules and their levels.
pub struct GenericConstrainer<DB: DatabaseLike> {
    tables: Vec<Registered<dyn TableRule<Database = DB>>>,
    columns: Vec<Registered<dyn ColumnRule<Database = DB>>>,
    foreign_keys: Vec<Registered<dyn ForeignKeyRule<Database = DB>>>,
    levels: Levels,
}

impl<DB: DatabaseLike> Default for GenericConstrainer<DB> {
    fn default() -> Self {
        Self {
            tables: Vec::new(),
            columns: Vec::new(),
            foreign_keys: Vec::new(),
            levels: Levels::default(),
        }
    }
}

impl<DB: DatabaseLike> GenericConstrainer<DB> {
    fn registered<R: Rule + ?Sized>(&self, rule: Box<R>, floor: Level) -> Registered<R> {
        let level = self.levels.resolve(rule.metadata(), floor);
        Registered { rule, floor, level }
    }

    pub(crate) fn register_table_rule_with_floor(
        &mut self,
        rule: Box<dyn TableRule<Database = DB>>,
        floor: Level,
    ) {
        let registered = self.registered(rule, floor);
        self.tables.push(registered);
    }

    pub(crate) fn register_column_rule_with_floor(
        &mut self,
        rule: Box<dyn ColumnRule<Database = DB>>,
        floor: Level,
    ) {
        let registered = self.registered(rule, floor);
        self.columns.push(registered);
    }

    pub(crate) fn register_foreign_key_rule_with_floor(
        &mut self,
        rule: Box<dyn ForeignKeyRule<Database = DB>>,
        floor: Level,
    ) {
        let registered = self.registered(rule, floor);
        self.foreign_keys.push(registered);
    }

    pub(crate) fn set_rule_level(&mut self, rule: &str, level: Level) {
        match self.levels.rules.iter_mut().find(|(name, _)| name == rule) {
            Some(entry) => entry.1 = level,
            None => self.levels.rules.push((rule.to_owned(), level)),
        }
        self.refresh();
    }

    pub(crate) fn set_group_level(&mut self, group: LintGroup, level: Level) {
        self.levels.groups[group.index()] = Some(level);
        self.refresh();
    }

    fn refresh(&mut self) {
        self.levels.refresh(&mut self.tables);
        self.levels.refresh(&mut self.columns);
        self.levels.refresh(&mut self.foreign_keys);
    }

    pub(crate) fn visit(
        &self,
        database: &DB,
        sink: &mut dyn FnMut(Error<DB>) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        for table in database.tables() {
            for entry in self
                .tables
                .iter()
                .filter(|entry| entry.level > Level::Allow)
            {
                let outcome = entry.rule.check_table(
                    database,
                    table,
                    &mut Reporter::new(database, entry.rule.metadata(), entry.level, sink),
                );
                settle(outcome, sink)?;
            }
            match table.columns(database) {
                Ok(columns) => {
                    for column in columns {
                        for entry in self
                            .columns
                            .iter()
                            .filter(|entry| entry.level > Level::Allow)
                        {
                            let outcome = entry.rule.check_column(
                                database,
                                column,
                                &mut Reporter::new(
                                    database,
                                    entry.rule.metadata(),
                                    entry.level,
                                    sink,
                                ),
                            );
                            settle(outcome, sink)?;
                        }
                    }
                }
                Err(error) => sink(Error::Lookup(error))?,
            }
            match table.foreign_keys(database) {
                Ok(foreign_keys) => {
                    for foreign_key in foreign_keys {
                        for entry in self
                            .foreign_keys
                            .iter()
                            .filter(|entry| entry.level > Level::Allow)
                        {
                            let outcome = entry.rule.check_foreign_key(
                                database,
                                foreign_key,
                                &mut Reporter::new(
                                    database,
                                    entry.rule.metadata(),
                                    entry.level,
                                    sink,
                                ),
                            );
                            settle(outcome, sink)?;
                        }
                    }
                }
                Err(error) => sink(Error::Lookup(error))?,
            }
        }
        ControlFlow::Continue(())
    }
}

/// Forwards a rule's lookup failure to the sink and propagates a halt.
fn settle<DB: DatabaseLike>(
    outcome: Result<(), Interruption>,
    sink: &mut dyn FnMut(Error<DB>) -> ControlFlow<()>,
) -> ControlFlow<()> {
    match outcome {
        Ok(()) => ControlFlow::Continue(()),
        Err(Interruption::Halted) => ControlFlow::Break(()),
        Err(Interruption::Lookup(error)) => sink(Error::Lookup(error)),
    }
}

impl<DB: DatabaseLike> Constrainer for GenericConstrainer<DB> {
    type Database = DB;

    fn constrainer(&self) -> &GenericConstrainer<DB> {
        self
    }

    fn constrainer_mut(&mut self) -> &mut GenericConstrainer<DB> {
        self
    }
}
