pub mod config;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{app::AppBuilder, error::Result, plugin::Plugin};
use config::MailerConfig;
use lettre::{transport::smtp::authentication::Credentials, Tokio1Executor};

pub type Mailer = lettre::AsyncSmtpTransport<Tokio1Executor>;
pub struct MailPlugin;

#[async_trait]
impl Plugin for MailPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<MailerConfig>(self)
            .context(format!("mail plugin config load failed"))
            .expect("mail plugin load failed");

        let mailer = Self::build_mailer(&config).expect("mail plugin load failed");

        app.add_component(mailer);
    }

    fn config_prefix(&self) -> &str {
        "mail"
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
