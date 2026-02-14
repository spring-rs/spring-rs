use serde::Deserialize;
use spring::component;
use spring::config::Configurable;
use spring::plugin::ComponentRegistry;
use spring::App;

// Define configuration
#[derive(Debug, Clone, Configurable, Deserialize)]
#[config_prefix = "database"]
struct DbConfig {
    host: String,
    port: u16,
}

// Define components
#[derive(Clone, Debug)]
struct DbConnection {
    #[allow(dead_code)]
    url: String,
}

#[derive(Clone, Debug)]
struct UserRepository {
    #[allow(dead_code)]
    db: DbConnection,
}

#[derive(Clone, Debug)]
struct UserService {
    #[allow(dead_code)]
    repo: UserRepository,
}

// Use #[component] macro to register components
#[component]
fn create_db_connection(
    spring::extractor::Config(config): spring::extractor::Config<DbConfig>,
) -> DbConnection {
    println!("Creating DbConnection with config: {:?}", config);
    DbConnection {
        url: format!("{}:{}", config.host, config.port),
    }
}

#[component]
fn create_user_repository(
    spring::extractor::Component(db): spring::extractor::Component<DbConnection>,
) -> UserRepository {
    println!("Creating UserRepository with db: {:?}", db);
    UserRepository { db }
}

#[component]
fn create_user_service(
    spring::extractor::Component(repo): spring::extractor::Component<UserRepository>,
) -> UserService {
    println!("Creating UserService with repo: {:?}", repo);
    UserService { repo }
}

#[tokio::main]
async fn main() {
    println!("Starting component-macro-example...");

    let app = App::new().build().await.expect("Failed to build app");

    // Get components
    let db = app
        .get_component::<DbConnection>()
        .expect("DbConnection not found");
    println!("Got DbConnection: {:?}", db);

    let repo = app
        .get_component::<UserRepository>()
        .expect("UserRepository not found");
    println!("Got UserRepository: {:?}", repo);

    let service = app
        .get_component::<UserService>()
        .expect("UserService not found");
    println!("Got UserService: {:?}", service);

    println!("All components registered successfully!");
}
