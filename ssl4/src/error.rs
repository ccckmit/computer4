use std::fmt;

#[derive(Debug)]
pub enum SslError {
    Io(std::io::Error),
    Tls(rustls::Error),
    CertGen(rcgen::RcgenError),
    PemParse(String),
    Other(String),
}

impl fmt::Display for SslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SslError::Io(e)       => write!(f, "IO 錯誤: {}", e),
            SslError::Tls(e)      => write!(f, "TLS 錯誤: {}", e),
            SslError::CertGen(e)  => write!(f, "憑證產生錯誤: {}", e),
            SslError::PemParse(s) => write!(f, "PEM 解析錯誤: {}", s),
            SslError::Other(s)    => write!(f, "錯誤: {}", s),
        }
    }
}

impl std::error::Error for SslError {}

impl From<std::io::Error> for SslError {
    fn from(e: std::io::Error) -> Self { SslError::Io(e) }
}

impl From<rustls::Error> for SslError {
    fn from(e: rustls::Error) -> Self { SslError::Tls(e) }
}

impl From<rcgen::RcgenError> for SslError {
    fn from(e: rcgen::RcgenError) -> Self { SslError::CertGen(e) }
}
