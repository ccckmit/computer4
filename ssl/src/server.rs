//! # TLS 伺服器模組
//!
//! 提供非同步 TLS 伺服器，建立在 `tokio-rustls` 之上。

use std::sync::Arc;
use std::net::SocketAddr;
use std::future::Future;

use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use rustls::{ServerConfig, server::NoClientAuth};

use crate::{cert::CertKeyPair, Result, SslError};

/// TLS 伺服器
pub struct TlsServer {
    listener: TcpListener,
    acceptor: TlsAcceptor,
    local_addr: SocketAddr,
}

impl TlsServer {
    /// 建立 TLS 伺服器
    pub async fn new(addr: &str, pair: CertKeyPair) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(pair.cert_chain, pair.private_key)
            .map_err(SslError::Tls)?;

        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;

        Ok(Self { listener, acceptor, local_addr })
    }

    /// 取得實際監聽位址
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// 開始接受連線，每條連線呼叫一次 `handler`
    pub async fn run<H, F>(&self, handler: H) -> Result<()>
    where
        H: Fn(TlsStream<TcpStream>, SocketAddr) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        let handler = Arc::new(handler);
        println!("🔒 TLS 伺服器啟動，監聽 {}", self.local_addr);

        loop {
            let (tcp_stream, peer_addr) = self.listener.accept().await?;
            let acceptor = self.acceptor.clone();
            let handler  = Arc::clone(&handler);

            tokio::spawn(async move {
                match acceptor.accept(tcp_stream).await {
                    Ok(tls_stream) => {
                        println!("✅ TLS 握手成功：{}", peer_addr);
                        handler(tls_stream, peer_addr).await;
                    }
                    Err(e) => {
                        eprintln!("❌ TLS 握手失敗 ({}): {}", peer_addr, e);
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::generate_self_signed;

    #[tokio::test]
    async fn test_server_binds() {
        let pair = generate_self_signed("localhost", &["localhost"]).unwrap();
        let server = TlsServer::new("127.0.0.1:0", pair).await.unwrap();
        let addr = server.local_addr();
        assert!(addr.port() > 0);
    }
}
