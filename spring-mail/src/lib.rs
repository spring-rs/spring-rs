//! [spring-mail](https://spring-rs.github.io/docs/plugins/spring-mail/)

pub mod config;

pub use lettre::message::*;
pub use lettre::AsyncTransport;
pub use lettre::Message;

use anyhow::Context;
use config::MailerConfig;
use lettre::{transport::smtp::authentication::Credentials, Tokio1Executor};
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::{app::AppBuilder, error::Result, plugin::Plugin};

pub type Mailer = lettre::AsyncSmtpTransport<Tokio1Executor>;

pub struct MailPlugin;

#[async_trait]
impl Plugin for MailPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<MailerConfig>()
            .expect("mail plugin config load failed");

        let mailer = Self::build_mailer(&config).expect("build mail plugin failed");

        app.add_component(mailer);
    }
}

impl MailPlugin {
    fn build_mailer(config: &MailerConfig) -> Result<Mailer> {
        let mut email_builder = if config.secure {
            Mailer::starttls_relay(&config.host)
                .with_context(|| format!("build mailer failed: {}", config.host))?
                .port(config.port)
        } else {
            Mailer::builder_dangerous(&config.host).port(config.port)
        };

        if let Some(auth) = config.auth.as_ref() {
            let credentials = Credentials::new(auth.user.clone(), auth.password.clone());
            email_builder = email_builder.credentials(credentials);
        }

        Ok(email_builder.build())
    }
}
