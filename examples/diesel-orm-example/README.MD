## Steps to run the examples
1. Choose one of the  databases from the supported list
    *   mysql
    *   postgres
    *   sqlite
2. Run the database migrations from the `diesel-orm-example ` directory  using the below commands. Make sure the replace the user:password and database url as required
    * diesel migration --database-url /tmp/data.db --migration-dir migrations-sqlite run
    *   diesel migration --database-url mysql://user:password@localhost:3306/users  --migration-dir ./migrations-mysql/   run
    *  diesel migration --migration-url postgresql://user:password@localhost:5432/postgres --migration-dir ./migrations-pg/ run

3. spring diesel plugin supports synchrnous diesel and async diesel-async. 
    * For async implementaiton, the following connection pools are supported
        * deadpool
        * bb8

    * For sync implementaiton, the following connection pools are supported
        * r2d2
4.  Updating your app.toml file - Uncomment the relevant section based on your database type(mysql, postgres, sqlite) , connection polling type(deadpool, bb8, r2d2) and implementation type (async and sync)
    
    Example app.toml entry for postgres,deadpool and async usage
    ```
    [diesel-async]
    uri = "postgresql://user:pass@localhost:5432/postgres"
    pool_type = "Deadpool"
    pool_config = { Deadpool = { max_size = 16,  create_timeout_in_ms=1000, recycle_timeout_in_ms=1000, wait_timeout_in_ms = 1000 } }
    ```

    Example app.toml entry for postgres,r2d2 and sync usage

    ```
    #[diesel-sync]
    uri = "postgresql://user:pass@localhost:5432/postgres"
    pool_type = "r2d2"
    pool_config = { R2d2 = { max_size = 16, min_idle= 4,  test_on_check_out = true, max_lifetime_in_ms = 300000, idle_timeout_in_ms=300000, connection_timeout_in_ms=5000} }
    ```

5.    Running the application
    
        * use cargo run command with --bin option as below for postgres deadpool async implementation 
            * cargo run --bin diesel-orm-sync-postgres-dp 
        * use cargo run command with --bin option as below for postgres r2d2 sync implementation 
            * cargo run --bin diesel-orm-sync-postgres-r2d2 
        * Check cargo.toml file other options available under [[bin]]

        * Verify the result by doing http get 
            * curl -v http://locahost:8080/users


    