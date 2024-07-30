use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct SeaOrmConfig {
    /// The URI for connecting to the database. For example:
    /// * Postgres: `postgres://root:12341234@localhost:5432/myapp_development`
    /// * Sqlite: `sqlite://db.sqlite?mode=rwc`
    pub uri: String,

    /// Enable `SQLx` statement logging
    #[serde(default)]
    pub enable_logging: bool,

    /// Minimum number of connections for a pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Maximum number of connections for a pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Set the timeout duration when acquiring a connection
    pub connect_timeout: Option<u64>,

    /// Set a maximum idle duration for individual connections.
    /// Any connection that remains in the idle queue longer than this will be closed.
    /// For usage-based database server billing, this can be a cost saver.
    pub idle_timeout: Option<u64>,

    /// Set the timeout for acquiring a connection
    pub acquire_timeout: Option<u64>,

    /// Truncate database when application loads. It will delete data from your
    /// tables. Commonly used in `test`.
    #[serde(default)]
    pub dangerously_truncate: bool,

    /// Recreate schema when application loads. Use it when you want to reset
    /// your database *and* structure (drop), this also deletes all of the data.
    /// Useful when you're just sketching out your project and trying out
    /// various things in development.
    #[serde(default)]
    pub dangerously_recreate: bool,
}

fn default_min_connections() -> u32 {
    1
}

fn default_max_connections() -> u32 {
    10
}
