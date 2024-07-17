pub mod config;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{app::App, plugin::Plugin};
use config::MailerConfig;
use lettre::{
    transport::smtp::{authentication::Credentials, commands::Mail},
    Tokio1Executor,
};

pub struct MailPlugin;

#[async_trait]
impl Plugin for MailPlugin {
    async fn build(&self, app: &mut App) {
        let config = app
            .get_config::<MailerConfig>(self)
            .context(format!("mail plugin config load failed"))
            .expect("mail plugin load failed");

        // Self::build_mailer(config);
    }

    fn config_prefix(&self) -> &str {
        "mail"
    }
}

impl MailPlugin {
    // fn build_mailer(config: MailerConfig) -> Result<EmailTransport, _> {
    //     let mut email_builder = if config.secure {
    //         lettre::AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
    //             .map_err(|error| {
    //                 tracing::error!(err.msg = %error, err.detail = ?error, "smtp_init_error");
    //                 Err(anyhow!("error initialize smtp mailer".to_string().into()))
    //             })?
    //             .port(config.port)
    //     } else {
    //         lettre::AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
    //             .port(config.port)
    //     };

    //     if let Some(auth) = config.auth.as_ref() {
    //         let credentials = Credentials::new(auth.user, auth.password);
    //         email_builder = email_builder.credentials(credentials);
    //     }

    //     Ok(EmailTransport::Smtp(email_builder.build()))
    // }
}

/// An enumeration representing the possible transport methods for sending
/// emails.
#[derive(Clone)]
pub enum EmailTransport {
    /// SMTP (Simple Mail Transfer Protocol) transport.
    Smtp(lettre::AsyncSmtpTransport<lettre::Tokio1Executor>),
    /// Test/stub transport for testing purposes.
    Test(lettre::transport::stub::StubTransport),
}
