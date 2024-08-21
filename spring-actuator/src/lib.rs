mod extractor;

use extractor::App;
use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::{app::AppBuilder, plugin::Plugin};
use spring_web::{
    axum::response::{IntoResponse, Json},
    axum::routing::get,
    extractor::Component,
    Router, Routers, WebConfigurator,
};

#[derive(Configurable)]
#[config_prefix = "actuator"]
pub struct ActuatorPlugin;

#[async_trait]
impl Plugin for ActuatorPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        app.add_router(actuator_router());
    }
}

fn actuator_router() -> Router {
    Router::new().nest(
        "/actuator",
        Router::new()
            .route("/health", get("ok"))
            .route("/endpoints", get(endpoints))
            .route("/components", get(components)),
    )
}

async fn endpoints(Component(routers): Component<Routers>) -> impl IntoResponse {
    let mut endpoints = vec![];
    for _r in routers {
        // TODO
        endpoints.push("");
    }
    Json(endpoints)
}

async fn components(app: App) -> impl IntoResponse {
    Json(app.get_components())
}
