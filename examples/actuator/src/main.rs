use autumn_actuator::ActuatorPlugin;
use autumn_boot::app::App;
use autumn_web::WebPlugin;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(ActuatorPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}
