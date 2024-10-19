//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-mail)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod config;

use lettre::address::Envelope;
pub use lettre::message::*;
use lettre::transport::smtp::response::Category;
use lettre::transport::smtp::response::Code;
use lettre::transport::smtp::response::Detail;
pub use lettre::transport::smtp::response::Response;
use lettre::transport::smtp::response::Severity;
pub use lettre::AsyncTransport;
pub use lettre::Message;

use anyhow::Context;
use config::MailerConfig;
use lettre::{transport::smtp::authentication::Credentials, Tokio1Executor};
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::{app::AppBuilder, error::Result, plugin::Plugin};

pub type TokioMailerTransport = lettre::AsyncSmtpTransport<Tokio1Executor>;
pub type StubMailerTransport = lettre::transport::stub::AsyncStubTransport;

#[derive(Clone)]
pub enum Mailer {
    Tokio(TokioMailerTransport),
    Stub(StubMailerTransport),
}

#[async_trait]
impl AsyncTransport for Mailer {
    type Ok = Response;
    type Error = spring::error::AppError;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok> {
        Ok(match self {
            Self::Tokio(tokio_transport) => tokio_transport
                .send_raw(envelope, email)
                .await
                .context("mailer send failed")?,
            Self::Stub(stub_transport) => {
                stub_transport
                    .send_raw(envelope, email)
                    .await
                    .context("stub mailer send failed")?;
                Response::new(
                    Code {
                        severity: Severity::PositiveCompletion,
                        category: Category::MailSystem,
                        detail: Detail::Zero,
                    },
                    vec!["stub mailer send success".to_string()],
                )
            }
        })
    }
}

pub struct MailPlugin;

#[async_trait]
impl Plugin for MailPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<MailerConfig>()
            .expect("mail plugin config load failed");

        let mailer = if config.stub {
            Mailer::Stub(StubMailerTransport::new_ok())
        } else {
            Mailer::Tokio(Self::build_mailer(&config).expect("build mail plugin failed"))
        };

        app.add_component(mailer);
    }
}

impl MailPlugin {
    fn build_mailer(config: &MailerConfig) -> Result<TokioMailerTransport> {
        let mut email_builder = if config.secure {
            TokioMailerTransport::starttls_relay(&config.host)
                .with_context(|| format!("build mailer failed: {}", config.host))?
                .port(config.port)
        } else {
            TokioMailerTransport::builder_dangerous(&config.host).port(config.port)
        };

        if let Some(auth) = config.auth.as_ref() {
            let credentials = Credentials::new(auth.user.clone(), auth.password.clone());
            email_builder = email_builder.credentials(credentials);
        }

        Ok(email_builder.build())
    }
}
