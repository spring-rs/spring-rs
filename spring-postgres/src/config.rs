use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct PgConfig {
    /// The URI for connecting to the postgres. For example:
    /// * postgresql: `postgres://root:12341234@localhost:5432/myapp_development`
    /// Please refer to [tokio_postgres::Config](https://docs.rs/tokio-postgres/latest/tokio_postgres/config/struct.Config.html) for details.
    pub connect: String,
}
