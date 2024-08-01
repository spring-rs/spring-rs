use async_trait::async_trait;
use autumn_boot::{app::AppBuilder, plugin::Plugin};
use autumn_web::{
    extractor::{App, Component},
    get,
    response::{IntoResponse, Json},
    Router, Routers, WebConfigurator,
};

pub struct ActuatorPlugin;

#[async_trait]
impl Plugin for ActuatorPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        app.add_router(actuator_router());
    }

    fn config_prefix(&self) -> &str {
        "actuator"
    }
}

fn actuator_router() -> Router {
    Router::new().nest(
        "/actuator",
        Router::new()
            .route("/health", get(health))
            .route("/endpoints", get(endpoints))
            .route("/components", get(components)),
    )
}

async fn health() -> impl IntoResponse {
    "ok"
}

async fn endpoints(Component(routers): Component<Routers>) -> impl IntoResponse {
    let mut endpoints = vec![];
    for r in routers {
        endpoints.push("");
    }
    Json(endpoints)
}

async fn components(app: App) -> impl IntoResponse {
    Json(app.get_components())
}
