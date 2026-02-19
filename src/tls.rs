//! TLS/HTTPS support using `rustls`.
//!
//! Provides [`TlsConfig`] for configuring HTTPS connections with
//! certificate and private key files in PEM format.

use std::fs::File;
use std::io::BufReader;
use std::net::TcpStream;
use std::sync::Arc;

use rustls::ServerConfig;
use rustls_pemfile;

use crate::error::{Error, Result};

/// TLS configuration for HTTPS support.
///
/// # Example
/// ```no_run
/// use mini_http::tls::TlsConfig;
///
/// let tls = TlsConfig::new("certs/cert.pem", "certs/key.pem").unwrap();
/// ```
pub struct TlsConfig {
    config: Arc<ServerConfig>,
}

impl TlsConfig {
    /// Create a TLS configuration from PEM certificate and private key files.
    ///
    /// # Errors
    /// Returns `Error::Tls` if the certificate or key files cannot be read or parsed.
    pub fn new(cert_path: &str, key_path: &str) -> Result<Self> {
        let cert_file = File::open(cert_path)
            .map_err(|e| Error::Tls(format!("Failed to open cert file: {}", e)))?;
        let key_file = File::open(key_path)
            .map_err(|e| Error::Tls(format!("Failed to open key file: {}", e)))?;

        let mut cert_reader = BufReader::new(cert_file);
        let mut key_reader = BufReader::new(key_file);

        let certs: Vec<_> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Tls(format!("Failed to parse certificates: {}", e)))?;

        if certs.is_empty() {
            return Err(Error::Tls("No certificates found in cert file".into()));
        }

        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| Error::Tls(format!("Failed to parse private key: {}", e)))?
            .ok_or_else(|| Error::Tls("No private key found in key file".into()))?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Error::Tls(format!("TLS configuration error: {}", e)))?;

        Ok(TlsConfig {
            config: Arc::new(config),
        })
    }

    /// Accept a TLS connection on a TCP stream.
    pub fn accept(&self, stream: TcpStream) -> Result<rustls::StreamOwned<rustls::ServerConnection, TcpStream>> {
        let conn = rustls::ServerConnection::new(Arc::clone(&self.config))
            .map_err(|e| Error::Tls(format!("Failed to create TLS connection: {}", e)))?;
        Ok(rustls::StreamOwned::new(conn, stream))
    }

    /// Get the underlying rustls ServerConfig.
    pub fn server_config(&self) -> Arc<ServerConfig> {
        Arc::clone(&self.config)
    }
}
