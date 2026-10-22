# SQL Rules

[![CI](https://github.com/earth-metabolome-initiative/sql-rules/workflows/Rust%20CI/badge.svg)](https://github.com/earth-metabolome-initiative/sql-rules/actions)
[![Security Audit](https://github.com/earth-metabolome-initiative/sql-rules/workflows/Security%20Audit/badge.svg)](https://github.com/earth-metabolome-initiative/sql-rules/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Codecov](https://codecov.io/gh/earth-metabolome-initiative/sql-rules/branch/main/graph/badge.svg)](https://codecov.io/gh/earth-metabolome-initiative/sql-rules)

[SQL-traits](https://github.com/earth-metabolome-initiative/sql-traits)-based higher-level rules for validating SQL schemas. These are general-purpose rules that are often desirable in SQL projects to enforce common standards (naming conventions, best practices) which are not strict SQL requirements but improve project quality and consistency.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sql_rules = "0.1.0"
```

## Usage

The library provides a `Constrainer` trait which applies registered `TableRule`, `ColumnRule`, and `ForeignKeyRule` implementations to a database schema.

### Using the Default Constrainer

The `DefaultConstrainer` comes pre-configured with a comprehensive set of common-sense rules.

```rust
use sql_rules::prelude::*;
use sqlparser::dialect::GenericDialect;
// ParserDB is available from sql-traits for testing and examples

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load or define your database schema
    let database = ParserDB::parse::<GenericDialect>("CREATE TABLE users (id INT PRIMARY KEY, name TEXT);")?;

    // 2. Create the default constrainer
    let constrainer = DefaultConstrainer::<ParserDB>::default();

    // 3. Validate the schema
    match constrainer.validate_schema(&database) {
        Ok(_) => println!("Schema is valid!"),
        Err(e) => eprintln!("Schema validation failed: {}", e),
    }

    Ok(())
}
```

### Custom Rules Configuration

You can select specific rules using `GenericConstrainer`.

```rust
use sql_rules::prelude::*;

let mut constrainer = GenericConstrainer::<ParserDB>::default();

// Register only specific rules
constrainer.register_table_rule(Box::new(SnakeCaseTableName::default()));
constrainer.register_column_rule(Box::new(LowercaseColumnName::default()));

// Use the constrainer...
```

## Available Rules

### Table Rules

| Rule | Description |
| :--- | :--- |
| `HasPrimaryKey` | Ensures every table has a primary key. |
| `LowercaseTableName` | Ensures table names are lowercase. |
| `NoForbiddenColumnInExtension` | Prevents forbidden columns in extended tables. |
| `NoNegationCheckRule` | Enforces that check constraints do not use negation (e.g. `NOT`). |
| `NoRustKeywordTableName` | Ensures table names are not reserved Rust keywords. |
| `NoTautologicalCheckRule` | Enforces that check constraints are not tautologies (always true). |
| `NonRedundantExtensionDag` | Ensures the table extension graph is free of redundancies. |
| `PluralTableName` | Ensures table names are plural. |
| `PoliciesRequireRowLevelSecurity` | Ensures that if a table has policies, RLS is enabled. |
| `SnakeCaseTableName` | Ensures table names follow `snake_case` convention. |
| `UniqueCheckRule` | Ensures check constraints are unique within a table. |
| `UniqueColumnNamesInExtensionGraph` | Ensures column names are unique across the table extension graph. |
| `UniqueForeignKey` | Ensures foreign keys are unique logic-wise per table. |
| `UniqueUniqueIndex` | Ensures unique indexes are not duplicated. |

### Column Rules

| Rule | Description |
| :--- | :--- |
| `LowercaseColumnName` | Ensures column names are lowercase. |
| `NoRustKeywordColumnName` | Ensures column names are not reserved Rust keywords. |
| `NonCompositePrimaryKeyNamedId` | Ensures non-composite primary keys are named `id`. |
| `PastTimeColumnRule` | Ensures time-related columns (ending in `_at`) have a check constraint ensuring past time. |
| `SingularColumnName` | Ensures column names are singular. |
| `SnakeCaseColumnName` | Ensures column names follow `snake_case` convention. |
| `TextualColumnRule` | Ensures textual columns are not empty and have length constraints. |

### Foreign Key Rules

| Rule | Description |
| :--- | :--- |
| `CompatibleForeignKey` | Ensures foreign keys types match their referenced primary keys. |
| `ExtensionForeignKeyOnDeleteCascade` | Ensures extension foreign keys have `ON DELETE CASCADE`. |
| `LowercaseForeignKeyName` | Ensures foreign key names are lowercase. |
| `NoRustKeywordForeignKeyName` | Ensures foreign key names are not reserved Rust keywords. |
| `PrimaryKeyReferenceEndsWithId` | Ensures foreign keys referencing a primary key end with `_id` suffix. |

## Contributing

If you can think of any rule that most SQL databases should enforce (or that represents a common best practice), please consider contributing it!
