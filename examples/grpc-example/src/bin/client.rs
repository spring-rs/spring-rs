use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use spring::{auto_config, plugin::MutableComponentRegistry, App};
use spring_web::{
    axum::response::IntoResponse, extractor::Component, get, WebConfigurator, WebPlugin,
};
use tonic::transport::Channel;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    let client = GreeterClient::connect("http://127.0.0.1:9090")
        .await
        .expect("failed to connect server, please start server first");
    App::new()
        .add_plugin(WebPlugin)
        .add_component(client)
        .run()
        .await
}

#[get("/hello")]
async fn hello(Component(mut client): Component<GreeterClient<Channel>>) -> impl IntoResponse {
    client
        .say_hello(tonic::Request::new(HelloRequest {
            name: "world".into(),
        }))
        .await
        .expect("failed to say hello")
        .into_inner()
        .message
}
