use anyhow::Context;
use spring::{auto_config, App};
use spring_mail::{header::ContentType, AsyncTransport, MailPlugin, Mailer, Message};
use spring_web::get;
use spring_web::{
    axum::response::{IntoResponse, Json},
    error::Result,
    extractor::Component,
    WebConfigurator, WebPlugin,
};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(MailPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/send")]
async fn send_mail(Component(mailer): Component<Mailer>) -> Result<impl IntoResponse> {
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("hff1996723@163.com".parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();
    let resp = mailer.send(email).await.context("send mail failed")?;
    Ok(Json(resp))
}
