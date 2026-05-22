//! # TLS 客戶端模組

use std::sync::Arc;
use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use rustls::{
    ClientConfig, RootCertStore, Certificate,
    client::{ServerCertVerifier, ServerCertVerified},
    ServerName, OwnedTrustAnchor,
};

use crate::{Result, SslError};

/// 憑證驗證模式
pub enum VerifyMode {
    /// 使用 Mozilla 內建根憑證
    SystemRoots,
    /// 使用自訂 CA 憑證（自簽名伺服器用）
    CustomCa(Vec<Certificate>),
    /// ⚠️ 停用憑證驗證（**僅限開發 / 測試**）
    DangerousNoVerify,
}

/// TLS 客戶端
pub struct TlsClient {
    connector:   TlsConnector,
    server_name: ServerName,
}

impl TlsClient {
    /// 建立 TLS 客戶端
    pub fn new(server_name: &str, mode: VerifyMode) -> Result<Self> {
        let config = match mode {
            VerifyMode::SystemRoots       => system_roots_config(),
            VerifyMode::CustomCa(certs)   => custom_ca_config(certs)?,
            VerifyMode::DangerousNoVerify => no_verify_config(),
        };

        let name = ServerName::try_from(server_name)
            .map_err(|_| SslError::Other(format!("無效的伺服器名稱: {}", server_name)))?;

        Ok(Self {
            connector: TlsConnector::from(Arc::new(config)),
            server_name: name,
        })
    }

    /// 連線至 TLS 伺服器
    pub async fn connect(&self, addr: SocketAddr) -> Result<TlsStream<TcpStream>> {
        let tcp = TcpStream::connect(addr).await?;
        let tls = self.connector
            .connect(self.server_name.clone(), tcp)
            .await
            .map_err(SslError::Io)?;
        Ok(tls)
    }

    /// 便利方法：連線、傳送訊息、接收回應
    pub async fn send_and_receive(
        &self,
        addr: SocketAddr,
        message: &[u8],
    ) -> Result<Vec<u8>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut stream = self.connect(addr).await?;
        stream.write_all(message).await?;
        stream.flush().await?;

        let mut buf = Vec::new();
        let mut tmp = [0u8; 4096];
        match stream.read(&mut tmp).await {
            Ok(0) => {}
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(e) => return Err(SslError::Io(e)),
        }
        Ok(buf)
    }
}

fn system_roots_config() -> ClientConfig {
    let mut root_store = RootCertStore::empty();
    root_store.add_server_trust_anchors(
        webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject, ta.spki, ta.name_constraints,
            )
        })
    );
    ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

fn custom_ca_config(ca_certs: Vec<Certificate>) -> Result<ClientConfig> {
    let mut root_store = RootCertStore::empty();
    for cert in ca_certs {
        root_store.add(&cert).map_err(SslError::Tls)?;
    }
    Ok(ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth())
}

fn no_verify_config() -> ClientConfig {
    struct NoVerifier;

    impl ServerCertVerifier for NoVerifier {
        fn verify_server_cert(
            &self,
            _: &Certificate,
            _: &[Certificate],
            _: &ServerName,
            _: &mut dyn Iterator<Item = &[u8]>,
            _: &[u8],
            _: std::time::SystemTime,
        ) -> std::result::Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }
    }

    ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth()
}
