use anyhow::Context;
use autumn::App;
use autumn_mail::{header::ContentType, AsyncTransport, MailPlugin, Mailer, Message};
use autumn_web::{
    error::Result,
    extractor::Component,
    get,
    response::{IntoResponse, Json},
    Router, WebConfigurator, WebPlugin,
};

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(MailPlugin)
        .add_plugin(WebPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new().route("/send", get(send_mail))
}

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
