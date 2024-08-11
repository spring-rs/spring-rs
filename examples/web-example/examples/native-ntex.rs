use ntex::web;

#[web::get("/")]
async fn index() -> impl web::Responder {
    "Hello, World!"
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::HttpServer::new(|| web::App::new().service(index))
        .bind(("127.0.0.1", 8082))?
        .run()
        .await
}