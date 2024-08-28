use anyhow::Context;
use instant_acme::{
    Account, AuthorizationStatus, ChallengeType, Identifier, NewAccount, NewOrder, OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use std::{path::PathBuf, time::Duration};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::config::AcmeServer;

pub async fn get_account(
    contact: &str,
    server: AcmeServer,
    acme_cert_dir: PathBuf,
) -> anyhow::Result<Account> {
    let credentials_file = acme_cert_dir.join("account.credentials");
    if credentials_file.exists() {
        let mut credentials_file = File::open(credentials_file)
            .await
            .context("account credentials open failed")?;
        let mut json = String::new();
        credentials_file.read_to_string(&mut json).await?;
        let credentials = serde_json::from_str(&json)?;
        Account::from_credentials(credentials)
            .await
            .context("Restore an existing account failed")
    } else {
        let (account, credentials) = Account::create(
            &NewAccount {
                contact: &[contact],
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            server.url(),
            None,
        )
        .await
        .context("request account failed")?;

        // write credentials
        let json = serde_json::to_string(&credentials)?;
        if let Some(parent) = credentials_file.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut credentials_file = fs::File::create(credentials_file).await?;
        credentials_file.write_all(json.as_bytes()).await?;

        Ok(account)
    }
}

pub async fn challenge(account: Account, domain: String) -> anyhow::Result<(String, String)> {
    // Create the ACME order based on the given domain names.
    // Note that this only needs an `&Account`, so the library will let you
    // process multiple orders in parallel for a single account.

    let identifier = Identifier::Dns(domain);
    let mut order = account
        .new_order(&NewOrder {
            identifiers: &[identifier],
        })
        .await
        .context("request order failed")?;

    let state = order.state();
    tracing::info!("order state: {:#?}", state);
    assert!(matches!(state.status, OrderStatus::Pending));

    // Pick the desired challenge type and prepare the response.

    let authorizations = order.authorizations().await.unwrap();
    let mut challenges = Vec::with_capacity(authorizations.len());
    for authz in &authorizations {
        match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            _ => todo!(),
        }

        // We'll use the DNS challenges for this example, but you could
        // pick something else to use here.

        let challenge = authz
            .challenges
            .iter()
            .find(|c| c.r#type == ChallengeType::Dns01)
            .ok_or_else(|| anyhow::anyhow!("no dns01 challenge found"))?;

        let Identifier::Dns(identifier) = &authz.identifier;

        println!("Please set the following DNS record then press the Return key:");
        println!(
            "_acme-challenge.{} IN TXT {}",
            identifier,
            order.key_authorization(challenge).dns_value()
        );
        std::io::stdin().read_line(&mut String::new()).unwrap();

        challenges.push((identifier, &challenge.url));
    }

    // Let the server know we're ready to accept the challenges.

    for (_, url) in &challenges {
        order.set_challenge_ready(url).await.unwrap();
    }

    // Exponentially back off until the order becomes ready or invalid.

    let mut tries = 1u8;
    let mut delay = Duration::from_millis(250);
    loop {
        tokio::time::sleep(delay).await;
        let state = order.refresh().await.unwrap();
        if let OrderStatus::Ready | OrderStatus::Invalid = state.status {
            tracing::info!("order state: {:#?}", state);
            break;
        }

        delay *= 2;
        tries += 1;
        match tries < 5 {
            true => tracing::info!(?state, tries, "order is not ready, waiting {delay:?}"),
            false => {
                tracing::error!(tries, "order is not ready: {state:#?}");
                return Err(anyhow::anyhow!("order is not ready"));
            }
        }
    }

    let state = order.state();
    if state.status != OrderStatus::Ready {
        return Err(anyhow::anyhow!(
            "unexpected order status: {:?}",
            state.status
        ));
    }

    let mut names = Vec::with_capacity(challenges.len());
    for (identifier, _) in challenges {
        names.push(identifier.to_owned());
    }

    // If the order is ready, we can provision the certificate.
    // Use the rcgen library to create a Certificate Signing Request.

    let mut params =
        CertificateParams::new(names.clone()).context("build CertificateParams failed")?;
    params.distinguished_name = DistinguishedName::new();
    let private_key = KeyPair::generate().context("generate key pair failed")?;
    let csr = params
        .serialize_request(&private_key)
        .context("build cert sign request failed")?;

    // Finalize the order and print certificate chain, private key and account credentials.

    order.finalize(csr.der()).await.unwrap();
    let cert_chain_pem = loop {
        match order.certificate().await.unwrap() {
            Some(cert_chain_pem) => break cert_chain_pem,
            None => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    };
    let private_key_pem = private_key.serialize_pem();
    tracing::info!("certficate chain:\n\n{}", cert_chain_pem);
    tracing::info!("private key:\n\n{}", private_key_pem);

    Ok((cert_chain_pem, private_key_pem))
}
