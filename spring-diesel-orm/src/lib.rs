//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-diesel-orm)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

//! ## spring-diesel-orm
//! Spring-rs component module that provides wrapper for `diesel` and `diesel-async` orm.
//! The module provides two  plugins that registers the connection pool as spring-rs component.
//! DieselSyncOrmPlugin provides wrapper for diesel sync implementation and supports r2d2 connection pool for postgres, mysql and sqlite
//! DieselASyncOrmPlugin provides wrapper for diesel-async implementation and supports
//! deadpool, bb8 connection pools for Postgres,mysql and sqlite databases.


pub mod config;

#[cfg(feature = "_diesel-async")]
pub mod diesel_async {

    use std::time::Duration;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_sqlite",
        feature = "_bb8"
    ))]
    use diesel_async::pooled_connection::bb8::Pool as Bb8Pool;
    use spring::app::AppBuilder;
    use spring::async_trait;
    use spring::config::ConfigRegistry;
    use spring::plugin::MutableComponentRegistry;
    use spring::plugin::Plugin;

    #[cfg(any(feature = "_diesel-sync", feature = "_sqlite"))]
    use diesel::SqliteConnection;

    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
    use diesel_async::pooled_connection::ManagerConfig;
    use diesel_async::pooled_connection::PoolError;

    #[cfg(feature = "_deadpool")]
    use diesel_async::AsyncConnection;

    #[cfg(feature = "_postgres")]
    use diesel_async::AsyncPgConnection;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_postgres",
        feature = "_deadpool"
    ))]
    use diesel_async::pooled_connection::deadpool::{BuildError, PoolBuilder, Pool as DeadPool};

    #[cfg(feature = "_sqlite")]
    use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;

    #[cfg(feature = "_mysql")]
    use diesel_async::AsyncMysqlConnection;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_postgres",
        feature = "_deadpool"
    ))]
    pub type  PgDeadPoolConnectionPool = DeadPool<AsyncPgConnection>;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_mysql",
        feature = "_deadpool"
    ))]
    pub type  MysqlDeadPoolConnectionPool = DeadPool<AsyncMysqlConnection>;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_sqlite",
        feature = "_deadpool"
    ))]
    pub type  SqliteDeadPoolConnectionPool = DeadPool<AsyncSqlLiteConnection>;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_postgres",
        feature = "_bb8"
    ))]
    pub type  PgBb8ConnectionPool = Bb8Pool<AsyncPgConnection>;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_mysql",
        feature = "_bb8"
    ))]
    pub type  MysqlBb8ConnectionPool = Bb8Pool<AsyncMysqlConnection>;

    #[cfg(all(
        feature = "_diesel-async",
        feature = "_sqlite",
        feature = "_bb8"
    ))]
    pub type  SqliteBb8ConnectionPool = Bb8Pool<AsyncSqlLiteConnection>;

    use crate::config::DieselAsyncOrmConfig;

    #[macro_export]
    macro_rules! configure_connections_deadpool {
    ($db:ident) => {
        paste::paste!{
            fn [<configure_async_connection_ $db _deadpool>](&self, config: &DieselAsyncOrmConfig, app_builder: &mut AppBuilder) {
                let pool = [<create_async_connection_ $db _deadpool>](config).expect("Failed to create connection pool");
                app_builder.add_component(pool);
            }
        }
    };
}

    #[macro_export]
    macro_rules! configure_connections_bb8 {
    ($db:ident) => {
        paste::paste!{
            async fn [<configure_async_connection_ $db _bb8>](&self, config: &DieselAsyncOrmConfig, app_builder: &mut AppBuilder) -> Result<(), PoolError>{
                let pool = [<create_async_connection_ $db _bb8>](config).await?;
                app_builder.add_component(pool);
                Ok(())
            }
        }
    };
}

    #[macro_export]
    macro_rules! create_async_connection_bb8 {
        ($connection:expr, $db: ident, $pool: ident) => {
            paste::paste!{
                async fn [<create_async_connection_ $db _bb8>](
                    config: &DieselAsyncOrmConfig,
                    ) -> std::result::Result<diesel_async::pooled_connection::bb8::Pool<$connection>, PoolError> {
                    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
                    let manager_config = create_manager_config(&config);
                    let manager: AsyncDieselConnectionManager<_> =
                        AsyncDieselConnectionManager::new_with_config(config.uri.clone(), manager_config);
                    let mut pool_builder = diesel_async::pooled_connection::bb8::Pool::builder();
                    pool_builder = [<configure_bb8_ $db >](pool_builder, config);
                    return pool_builder.build(manager).await;
                }
            }
        };
    }

    #[macro_export] // Makes the macro available throughout the crate
    macro_rules! deadpool_timeout {
        ($connection:expr, $db:ident) =>{
            paste::paste! {
                fn [<set_timeouts_deadpool _$db>](mut pool_builder: PoolBuilder<$connection>, orm_config: &DieselAsyncOrmConfig) -> PoolBuilder<$connection>{
                    if let Some($crate::config::PoolConfig::Deadpool(ref config)) = &orm_config.pool_config {
                        pool_builder = pool_builder.runtime(deadpool::Runtime::Tokio1);
                        if let Some(value) = config.create_timeout_in_ms {
                            pool_builder = pool_builder.create_timeout(Some(Duration::from_millis(value)));
                        }
                        if let Some(value) = config.reycle_timeout_in_ms {
                            pool_builder = pool_builder.recycle_timeout(Some(Duration::from_millis(value)));
                        }
                        if let Some(value) = config.wait_timeout_in_ms {
                            pool_builder = pool_builder.wait_timeout(Some(Duration::from_millis(value)));
                        }
                    }
                    return pool_builder;
                }
            }
        }
}

    #[macro_export]
    macro_rules! create_async_connection_deadpool {
    ($connection:expr, $db: ident, $pool: ident) => {
        paste::paste!{
            fn [<create_async_connection_ $db _deadpool>](
                config: &DieselAsyncOrmConfig,
                ) -> std::result::Result<diesel_async::pooled_connection::deadpool::Pool<$connection>, BuildError> {
                use diesel_async::pooled_connection::AsyncDieselConnectionManager;
                let manager_config = create_manager_config(&config);
                let manager =
                    AsyncDieselConnectionManager::new_with_config(config.uri.clone(), manager_config);
                let mut pool_builder = diesel_async::pooled_connection::deadpool::Pool::builder(manager);
                if let Some($crate::config::PoolConfig::Deadpool(deadpool_config)) = &config.pool_config {
                        pool_builder = pool_builder.max_size(deadpool_config.max_connections);
                        pool_builder = [<set_timeouts_deadpool_ $db>](pool_builder, config);
                }
                pool_builder.build()
            }
        }
    };
}

    #[macro_export] // Makes the macro available throughout the crate
    macro_rules! configure_bb8 {
        ($connection:expr, $db:ident) =>{
            paste::paste! {
                fn [<configure_bb8 _$db>](mut pool_builder: bb8::Builder<AsyncDieselConnectionManager<$connection>>, orm_config: &DieselAsyncOrmConfig) -> bb8::Builder<AsyncDieselConnectionManager<$connection>> {
                    
                    if let Some($crate::config::PoolConfig::Bb8(config)) = &orm_config.pool_config {
                        if let Some(value) = config.max_size {
                            pool_builder = pool_builder.max_size(value);
                        }

                        if let Some(value) = config.min_idle {
                            pool_builder = pool_builder.min_idle(value);
                        }

                        if let Some(value) = config.test_on_check_out {
                            pool_builder = pool_builder.test_on_check_out(value);
                        }

                        if let Some(value) = config.max_lifetime_in_ms {
                            pool_builder = pool_builder.max_lifetime(Some(Duration::from_millis(value)));
                        }

                        if let Some(value) = config.idle_timeout_in_ms {
                            pool_builder = pool_builder.idle_timeout(Some(Duration::from_millis(value)));
                        }

                        if let Some(value) = config.connection_timeout_in_ms {
                            pool_builder = pool_builder.connection_timeout(Duration::from_millis(value));
                        }

                        if let Some(value) = config.retry_connection {
                            pool_builder = pool_builder.retry_connection(value);
                        }

                        if let Some(value) = config.reaper_rate_in_ms {
                            pool_builder = pool_builder.reaper_rate(Duration::from_millis(value));
                        }

                        if let Some(value) = &config.queue_strategy {
                            match value{
                                $crate::config::QueueStrategy::Fifo => {
                                    pool_builder = pool_builder.queue_strategy(bb8::QueueStrategy::Fifo);
                                }
                                $crate::config::QueueStrategy::Lifo => {
                                    pool_builder = pool_builder.queue_strategy(bb8::QueueStrategy::Lifo);
                                }
                            }
                        }
                    }
                    return pool_builder;
                }
            }
        }
    }

    impl DieselAsyncOrmPlugin {
        #[cfg(all(feature = "_postgres", feature = "_deadpool"))]
        configure_connections_deadpool!(pg);

        #[cfg(all(feature = "_mysql", feature = "_deadpool"))]
        configure_connections_deadpool!(mysql);

        #[cfg(all(feature = "_sqlite", feature = "_deadpool"))]
        configure_connections_deadpool!(sqlite);

        #[cfg(all(feature = "_postgres", feature = "_bb8"))]
        configure_connections_bb8!(pg);

        #[cfg(all(feature = "_mysql", feature = "_bb8"))]
        configure_connections_bb8!(mysql);

        #[cfg(all(feature = "_sqlite", feature = "_bb8"))]
        configure_connections_bb8!(sqlite);
    }

    #[cfg(feature = "_deadpool")]
    fn create_manager_config<C>(config: &DieselAsyncOrmConfig) -> ManagerConfig<C>
    where
        C: AsyncConnection + 'static,
    {
        use diesel_async::pooled_connection::ManagerConfig;

        let mut manager_config = ManagerConfig::default();
        if let Some(recycle_method) = &config.connection_recycle_method {
            use crate::config::RecycleMethod;

            match recycle_method {
                RecycleMethod::Fast => {
                    manager_config.recycling_method =
                        diesel_async::pooled_connection::RecyclingMethod::Fast;
                }
                RecycleMethod::Verified(option) => {
                    if let Some(sql) = option.to_owned() {
                        let str = Box::leak(Box::new(sql));
                        manager_config.recycling_method =
                            diesel_async::pooled_connection::RecyclingMethod::CustomQuery(
                                std::borrow::Cow::Borrowed(str),
                            )
                    } else {
                        manager_config.recycling_method =
                            diesel_async::pooled_connection::RecyclingMethod::Verified;
                    }
                }
            }
        }
        manager_config
    }

    #[cfg(feature = "_sqlite")]
    pub type AsyncSqlLiteConnection = SyncConnectionWrapper<SqliteConnection>;

    #[cfg(all(feature = "_postgres", feature = "_deadpool"))]
    deadpool_timeout!(AsyncPgConnection, pg);

    #[cfg(all(feature = "_mysql", feature = "_deadpool"))]
    deadpool_timeout!(AsyncMysqlConnection, mysql);

    #[cfg(all(feature = "_sqlite", feature = "_deadpool"))]
    deadpool_timeout!(AsyncSqlLiteConnection, sqlite);

    #[cfg(all(feature = "_postgres", feature = "_deadpool"))]
    create_async_connection_deadpool!(AsyncPgConnection, pg, deadpool);

    #[cfg(all(feature = "_mysql", feature = "_deadpool"))]
    create_async_connection_deadpool!(AsyncMysqlConnection, mysql, deadpool);

    #[cfg(all(feature = "_sqlite", feature = "_deadpool"))]
    create_async_connection_deadpool!(AsyncSqlLiteConnection, sqlite, deadpool);

    #[cfg(all(feature = "_postgres", feature = "_bb8"))]
    create_async_connection_bb8!(AsyncPgConnection, pg, bb8);

    #[cfg(all(feature = "_mysql", feature = "_bb8"))]
    create_async_connection_bb8!(AsyncMysqlConnection, mysql, deadpool);

    #[cfg(all(feature = "_sqlite", feature = "_bb8"))]
    create_async_connection_bb8!(AsyncSqlLiteConnection, sqlite, deadpool);

    #[cfg(all(feature = "_postgres", feature = "_bb8"))]
    configure_bb8!(AsyncPgConnection, pg);

    #[cfg(all(feature = "_mysql", feature = "_bb8"))]
    configure_bb8!(AsyncMysqlConnection, mysql);

    #[cfg(all(feature = "_sqlite", feature = "_bb8"))]
    configure_bb8!(AsyncSqlLiteConnection, sqlite);

    #[cfg(feature = "_diesel-async")]
    pub struct DieselAsyncOrmPlugin;

    #[cfg(feature = "_diesel-async")]
    #[async_trait]
    impl Plugin for DieselAsyncOrmPlugin {
        async fn build(&self, app: &mut AppBuilder) {
            use crate::config::DieselAsyncOrmConfig;
            use crate::config::PoolType;

            let config = app
                .get_config::<DieselAsyncOrmConfig>()
                .expect("diesels-orm plugin config load failed");

            #[cfg(all(feature = "_postgres", feature = "_deadpool"))]
            if config.pool_type == PoolType::Deadpool && config.uri.contains("postgres") {
                self.configure_async_connection_pg_deadpool(&config, app);
            }

            #[cfg(all(feature = "_mysql", feature = "_deadpool"))]
            if config.uri.starts_with("mysql") {
                self.configure_async_connection_mysql_deadpool(&config, app);
            }

            #[cfg(all(feature = "_sqlite", feature = "_deadpool"))]
            if config.uri.starts_with("file")
                || config.uri.starts_with("/")
                || config.uri.starts_with("sqlite")
            {
                self.configure_async_connection_sqlite_deadpool(&config, app);
            }

            #[cfg(all(feature = "_postgres", feature = "_bb8"))]
            if config.pool_type == PoolType::Bb8 && config.uri.starts_with("postgres") {
                self.configure_async_connection_pg_bb8(&config, app)
                    .await
                    .expect("Failed to configure PostgreSQL BB8 connection");
            }

            #[cfg(all(feature = "_mysql", feature = "_bb8"))]
            if config.pool_type == PoolType::Bb8 && config.uri.starts_with("mysql") {
                self.configure_async_connection_mysql_bb8(&config, app)
                    .await
                    .expect("Failed to configure MySQL BB8 connection");
            }

            #[cfg(all(feature = "_sqlite", feature = "_bb8"))]
            if config.pool_type == PoolType::Bb8
                && (config.uri.starts_with("file")
                    || config.uri.starts_with("/")
                    || config.uri.starts_with("sqlite"))
            {
                self.configure_async_connection_sqlite_bb8(&config, app)
                    .await
                    .expect("Failed to configure SQLite BB8 connection");
            }
        }
    }
}

#[cfg(feature = "_diesel-sync")]
pub mod diesel_sync {

    use spring::{app::AppBuilder, async_trait, plugin::Plugin};
    use std::time::Duration;

    use crate::config::DieselSyncOrmConfig;
    use diesel::r2d2::{ConnectionManager, Pool as R2d2Pool, R2D2Connection};
    #[cfg(feature = "_mysql")]
    use diesel::MysqlConnection;
    #[cfg(feature = "_postgres")]
    use diesel::PgConnection;
    use diesel::SqliteConnection;
    use spring::plugin::MutableComponentRegistry;
    use diesel::r2d2::PoolError;
    
    #[cfg(feature = "_postgres")]
    pub type  PgR2d2ConnectionPool = R2d2Pool<ConnectionManager<PgConnection>>;

    #[cfg(feature = "_mysql")]
    pub type  MysqlR2d2ConnectionPool = R2d2Pool<ConnectionManager<MysqlConnection>>;

    #[cfg(feature = "_sqlite")]
    pub type  SqliteR2d2ConnectionPool = R2d2Pool<ConnectionManager<SqliteConnection>>;

    

    fn create_async_connection_r2d2<C>(
        config: &DieselSyncOrmConfig,
    ) -> Result<R2d2Pool<ConnectionManager<C>>, PoolError>
    where
        C: R2D2Connection + 'static,
    {
        let manager = ConnectionManager::<C>::new(config.uri.clone());
        let mut builder = R2d2Pool::builder();
        let pool_config = config.pool_config.as_ref();

        if let Some(pool_config) = pool_config {
            if let Some(max_size) = pool_config.max_size {
                builder = builder.max_size(max_size);
            }

            builder = builder.min_idle(pool_config.min_idle);

            if let Some(test_on_check_out) = pool_config.test_on_check_out {
                builder = builder.test_on_check_out(test_on_check_out);
            }

            if let Some(max_lifetime) = pool_config.max_lifetime_in_ms {
                builder = builder.max_lifetime(Some(Duration::from_millis(max_lifetime)));
            }

            if let Some(idle_timeout) = pool_config.idle_timeout_in_ms {
                builder = builder.idle_timeout(Some(Duration::from_millis(idle_timeout)));
            }

            if let Some(connection_timeout) = pool_config.connection_timeout_in_ms {
                builder = builder.connection_timeout(Duration::from_millis(connection_timeout));
            }

            if let Some(test_on_check_out) = pool_config.test_on_check_out {
                builder = builder.test_on_check_out(test_on_check_out);
            }

            if let Some(max_size) = pool_config.max_size {
                builder = builder.max_size(max_size);
            }

            if let Some(idle_timeout) = pool_config.idle_timeout_in_ms {
                builder = builder.idle_timeout(Some(Duration::from_millis(idle_timeout)));
            }
        }

        builder.build(manager)
    }

    pub struct DieselSyncOrmPlugin;

    #[cfg(feature = "_diesel-sync")]
    #[async_trait]
    impl Plugin for DieselSyncOrmPlugin {
        async fn build(&self, app: &mut AppBuilder) {
            use spring::config::ConfigRegistry;

            use crate::config::DieselSyncOrmConfig;

            let config = app
                .get_config::<DieselSyncOrmConfig>()
                .expect("diesels-orm plugin config load failed");

            #[cfg(feature = "_postgres")]
            if config.uri.starts_with("postgres") {
                let connection = create_async_connection_r2d2::<PgConnection>(&config)
                    .expect("Failed to create postgres connection pool");
                app.add_component(connection);
            }

            #[cfg(feature = "_mysql")]
            if config.uri.starts_with("mysql") {
                let connection = create_async_connection_r2d2::<MysqlConnection>(&config)
                    .expect("Failed to create mysql connection pool");
                app.add_component(connection);
            }

            #[cfg(feature = "_sqlite")]
            if config.uri.starts_with("file")
                || config.uri.starts_with("/")
                || config.uri.starts_with("sqlite")
            {
                let connection = create_async_connection_r2d2::<SqliteConnection>(&config)
                    .expect("Failed to create sqlite connection pool");
                app.add_component(connection);
            }
        }
    }
}
