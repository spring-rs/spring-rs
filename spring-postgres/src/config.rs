use schemars::JsonSchema;
use serde::Deserialize;
use spring::config::Configurable;

#[derive(Debug, Configurable, Clone, JsonSchema, Deserialize)]
#[config_prefix = "postgres"]
pub struct PgConfig {
    /// The URI for connecting to the postgres. For example:
    /// * postgresql: `postgres://root:12341234@localhost:5432/myapp_development`
    /// Please refer to [tokio_postgres::Config] for details.
    pub connect: String,
}
