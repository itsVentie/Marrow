use std::sync::Arc;
use anyhow::Context;
use quinn::ServerConfig;

pub fn make_server_config() -> anyhow::Result<ServerConfig> {
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .context("Failed to build cert params")?
        .self_signed(&key_pair)
        .context("Failed to self-sign certificate")?;

    let cert_der = rustls_pki_types::CertificateDer::from(cert.der().to_vec());
    let key_der = rustls_pki_types::PrivateKeyDer::Pkcs8(
        rustls_pki_types::PrivatePkcs8KeyDer::from(key_pair.serialize_der()),
    );

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .context("Failed to build rustls ServerConfig")?;

    Ok(ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)
            .context("Failed to build QUIC server config")?,
    )))
}