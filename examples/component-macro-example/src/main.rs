use spring::config::Configurable;
use spring::plugin::ComponentRegistry;
use spring::component;
use spring::App;
use serde::Deserialize;

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
    url: String,
}

#[derive(Clone, Debug)]
struct UserRepository {
    db: DbConnection,
}

#[derive(Clone, Debug)]
struct UserService {
    repo: UserRepository,
}

// Use #[component] macro to register components
#[component]
fn create_db_connection(
    spring::config::Config(config): spring::config::Config<DbConfig>,
) -> DbConnection {
    println!("Creating DbConnection with config: {:?}", config);
    DbConnection {
        url: format!("{}:{}", config.host, config.port),
    }
}

#[component]
fn create_user_repository(
    spring::plugin::Component(db): spring::plugin::Component<DbConnection>,
) -> UserRepository {
    println!("Creating UserRepository with db: {:?}", db);
    UserRepository { db }
}

#[component]
fn create_user_service(
    spring::plugin::Component(repo): spring::plugin::Component<UserRepository>,
) -> UserService {
    println!("Creating UserService with repo: {:?}", repo);
    UserService { repo }
}

#[tokio::main]
async fn main() {
    println!("Starting component-macro-example...");
    
    let app = App::new()
        .build()
        .await
        .expect("Failed to build app");
    
    // Get components
    let db = app.get_component::<DbConnection>().expect("DbConnection not found");
    println!("Got DbConnection: {:?}", db);
    
    let repo = app.get_component::<UserRepository>().expect("UserRepository not found");
    println!("Got UserRepository: {:?}", repo);
    
    let service = app.get_component::<UserService>().expect("UserService not found");
    println!("Got UserService: {:?}", service);
    
    println!("All components registered successfully!");
}
