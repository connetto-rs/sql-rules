//! Submodule defining traits used for SQL schema rules.

mod table_rule;
pub use table_rule::TableRule;
mod column_rule;
pub use column_rule::ColumnRule;
pub mod constrainer;
pub use constrainer::{Constrainer, DefaultConstrainer, GenericConstrainer};
mod foreign_key_rule;
pub use foreign_key_rule::ForeignKeyRule;
mod reporter;
pub use reporter::Reporter;
