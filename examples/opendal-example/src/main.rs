use spring::{auto_config, get, post, App};
use spring_opendal::{Op, OpenDALPlugin};
use spring_web::extractor::Component;
use spring_web::http::StatusCode;
use spring_web::response::IntoResponse;
use spring_web::{WebConfigurator, WebPlugin};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(OpenDALPlugin)
        .run()
        .await
}

#[get("/")]
async fn index() -> impl IntoResponse {
    "Hello, OpenDAL!"
}

const FILE_NAME: &str = "test.spring";

#[get("/read")]
async fn read_file(Component(op): Component<Op>) -> impl IntoResponse {
    let b = op.is_exist(FILE_NAME).await.unwrap();
    if !b {
        return (StatusCode::NOT_FOUND, "File not found".to_string());
    }
    let bf = op.read_with(FILE_NAME).await.unwrap();
    (StatusCode::OK, String::from_utf8(bf.to_vec()).unwrap())
}

#[get("/info")]
async fn stat_file(Component(op): Component<Op>) -> impl IntoResponse {
    (StatusCode::OK, format!("{:?}", op.info()))
}

#[post("/write")]
async fn write_file(Component(op): Component<Op>) -> impl IntoResponse {
    match op.write(FILE_NAME, "Hello, World!").await {
        Ok(_) => (StatusCode::OK, "Write file success".to_string()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
