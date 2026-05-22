/// ssl4 — 簡易 SSL/TLS 套件
///
/// 提供三大功能模組：
///   - `cert`   : 憑證產生與載入
///   - `server` : TLS 伺服器
///   - `client` : TLS 客戶端
pub mod cert;
pub mod server;
pub mod client;
pub mod error;

pub use error::SslError;
pub type Result<T> = std::result::Result<T, SslError>;
