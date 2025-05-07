use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use spring::App;
use spring_web::WebPlugin;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() {
    App::new().add_plugin(WebPlugin).run().await
}
