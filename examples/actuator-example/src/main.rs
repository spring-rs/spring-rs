use spring::App;
use spring_actuator::ActuatorPlugin;
use spring_web::WebPlugin;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(ActuatorPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}
