use schemars::JsonSchema;
use serde::Deserialize;
use spring::config::Configurable;

#[cfg(feature = "_diesel-async")]
spring::submit_config_schema!("diesel-async-orm", DieselAsyncOrmConfig);

#[cfg(feature = "_diesel-sync")]
spring::submit_config_schema!("diesel-sync-orm", DieselSyncOrmConfig);

#[cfg(feature = "_diesel-sync")]
#[derive(Debug, Configurable, Clone, JsonSchema, Deserialize)]
#[config_prefix = "diesel-sync"]
pub(crate) struct DieselSyncOrmConfig {
    /// The URI for connecting to the database. For example:
    /// * Postgres: `postgres://root:12341234@localhost:5432/myapp_development`
    /// * Sqlite: `sqlite://db.sqlite?mode=rwc`
    pub uri: String,

    pub pool_config: Option<R2d2PoolConfig>,

    //pub connection_recycle_method: Option<RecycleMethod>,
}


#[cfg(feature = "_diesel-async")]
#[derive(Debug, Configurable, Clone, JsonSchema, Deserialize)]
#[config_prefix = "diesel-async"]
pub(crate) struct DieselAsyncOrmConfig {
    /// The URI for connecting to the database. For example:
    /// * Postgres: `postgres://root:12341234@localhost:5432/myapp_development`
    /// * Sqlite: `sqlite://db.sqlite?mode=rwc`
    pub uri: String,

    pub connection_recycle_method: Option<RecycleMethod>,

    pub pool_config: Option<PoolConfig>,

    pub pool_type: PoolType,
}


#[cfg(feature = "_diesel-async")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) enum PoolConfig{
    Deadpool(DeadPoolConfig),
    Bb8(Bb8PoolConfig),
}

#[cfg(feature = "_diesel-async")]
#[derive(Debug, Clone, JsonSchema, Deserialize, PartialEq)]
pub(crate) enum PoolType {
    Deadpool,
    Bb8,
}

#[cfg(feature = "_diesel-async")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) struct DeadPoolConfig{

    /// Maximum number of connections for a pool
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /* #[serde(default = "default_min_connections")]
    pub min_connections: usize, */

    /// Set the timeout duration when creating a connection
    pub create_timeout_in_ms: Option<u64>,

    /// Set the timeout for acquiring a connection
    pub wait_timeout_in_ms: Option<u64>,

    pub reycle_timeout_in_ms: Option<u64>,

}

#[cfg(feature = "_diesel-async")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate)struct Bb8PoolConfig{

    pub max_size: Option<u32>,

    pub min_idle: Option<u32>,

    pub test_on_check_out: Option<bool>,

    pub max_lifetime_in_ms: Option<u64>,

    /// The duration, if any, after which idle_connections in excess of `min_idle` are closed.
    pub idle_timeout_in_ms: Option<u64>,

    /// The duration to wait to start a connection before giving up.
    pub connection_timeout_in_ms: Option<u64>,

    /// Enable/disable automatic retries on connection creation.
    pub retry_connection: Option<bool>,
    
    /// The time interval used to wake up and reap connections.
    pub(crate) reaper_rate_in_ms: Option<u64>,

    /// Queue strategy (FIFO or LIFO)
    pub(crate) queue_strategy: Option<QueueStrategy>,
}

#[cfg(feature = "_diesel-async")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub enum QueueStrategy{
    FIFO,
    LIFO,
}


#[cfg(feature = "_diesel-async")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) enum RecycleMethod{
    Fast,
    Verified(Option<String>),    
}

#[cfg(feature = "_diesel-sync")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate)struct R2d2PoolConfig{

    pub max_size: Option<u32>,

    pub min_idle: Option<u32>,

    pub test_on_check_out: Option<bool>,

    pub max_lifetime_in_ms: Option<u64>,

    /// The duration, if any, after which idle_connections in excess of `min_idle` are closed.
    pub idle_timeout_in_ms: Option<u64>,

    /// The duration to wait to start a connection before giving up.
    pub connection_timeout_in_ms: Option<u64>,
}

#[cfg(feature = "_diesel-async")]
fn default_max_connections() -> usize {
    16
}