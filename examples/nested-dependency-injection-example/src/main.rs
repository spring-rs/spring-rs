use schemars::JsonSchema;
use serde::Deserialize;
use spring::{
    auto_config,
    config::Configurable,
    plugin::service::Service,
    App,
};
use spring_web::get;
use spring_web::{
    axum::response::IntoResponse, error::Result, extractor::Component, WebConfigurator, WebPlugin,
};


// Main function entry
#[auto_config(WebConfigurator)] // auto config web router
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[derive(Clone, Configurable, JsonSchema, Deserialize)]
#[config_prefix = "user"]
struct UserConfig {
    username: String,
    project: String,
    #[serde(default)]
    init_count: i32,
}


#[derive(Clone, Service)]
struct UserService {
    #[inject(config)]
    config: UserConfig,
}

#[derive(Clone, Service)]
struct BetterUserService {
    #[inject(component)]
    user_service: UserService,
}

impl BetterUserService{
    fn get_info(&self) -> String {
        self.user_service.config.username.clone()
    }
}


#[get("/")]
async fn hello(Component(better_user): Component<BetterUserService>) -> Result<impl IntoResponse> {
    Ok(better_user.get_info())
}
