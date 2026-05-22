//! # 憑證模組
//!
//! 支援：
//! - 自動產生自簽名 (self-signed) 憑證與私鑰
//! - 從 PEM 載入憑證與私鑰
//! - 將憑證存成 PEM 格式

use rcgen::{CertificateParams, DistinguishedName, DnType, SanType, Certificate};
use rustls::{Certificate as RustlsCert, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;

use crate::{Result, SslError};

/// 憑證 + 私鑰組合
pub struct CertKeyPair {
    /// DER 格式憑證串列
    pub cert_chain: Vec<RustlsCert>,
    /// PKCS8 私鑰
    pub private_key: PrivateKey,
    /// PEM 格式憑證（供匯出使用）
    pub cert_pem: String,
    /// PEM 格式私鑰（供匯出使用）
    pub key_pem: String,
}

/// 產生自簽名憑證
///
/// # 參數
/// - `common_name`: 例如 `"localhost"`
/// - `sans`: Subject Alternative Names，例如 `["localhost", "127.0.0.1"]`
///
/// # 範例
/// ```no_run
/// use simple_ssl::cert::generate_self_signed;
/// let pair = generate_self_signed("localhost", &["localhost", "127.0.0.1"]).unwrap();
/// println!("{}", pair.cert_pem);
/// ```
pub fn generate_self_signed(common_name: &str, sans: &[&str]) -> Result<CertKeyPair> {
    let mut params = CertificateParams::default();

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, common_name);
    dn.push(DnType::OrganizationName, "Simple SSL");
    dn.push(DnType::CountryName, "TW");
    params.distinguished_name = dn;

    params.subject_alt_names = sans
        .iter()
        .map(|s| {
            if let Ok(ip) = s.parse::<std::net::IpAddr>() {
                SanType::IpAddress(ip)
            } else {
                SanType::DnsName(s.to_string())
            }
        })
        .collect();

    let cert = Certificate::from_params(params)?;

    let cert_pem = cert.serialize_pem()?;
    let key_pem  = cert.serialize_private_key_pem();

    let cert_der = cert.serialize_der()?;
    let key_der  = cert.serialize_private_key_der();

    Ok(CertKeyPair {
        cert_chain:  vec![RustlsCert(cert_der)],
        private_key: PrivateKey(key_der),
        cert_pem,
        key_pem,
    })
}

/// 從 PEM 字串載入憑證與私鑰
pub fn load_from_pem(cert_pem: &str, key_pem: &str) -> Result<CertKeyPair> {
    let mut cert_reader = BufReader::new(cert_pem.as_bytes());
    let cert_chain: Vec<RustlsCert> = certs(&mut cert_reader)
        .map_err(|e| SslError::Io(e))?
        .into_iter()
        .map(RustlsCert)
        .collect();

    if cert_chain.is_empty() {
        return Err(SslError::PemParse("找不到憑證".to_string()));
    }

    let mut key_reader = BufReader::new(key_pem.as_bytes());
    let keys = pkcs8_private_keys(&mut key_reader)
        .map_err(|e| SslError::Io(e))?;

    if keys.is_empty() {
        return Err(SslError::PemParse("找不到 PKCS8 私鑰".to_string()));
    }

    Ok(CertKeyPair {
        cert_chain,
        private_key: PrivateKey(keys[0].clone()),
        cert_pem: cert_pem.to_string(),
        key_pem: key_pem.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_self_signed() {
        let pair = generate_self_signed("localhost", &["localhost", "127.0.0.1"])
            .expect("應能產生憑證");
        assert!(pair.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(pair.key_pem.contains("BEGIN PRIVATE KEY") || pair.key_pem.contains("BEGIN EC PRIVATE KEY"));
        assert!(!pair.cert_chain.is_empty());
    }

    #[test]
    fn test_roundtrip_pem() {
        let original = generate_self_signed("test.local", &["test.local"]).unwrap();
        let loaded   = load_from_pem(&original.cert_pem, &original.key_pem)
            .expect("應能從 PEM 載入");
        assert!(!loaded.cert_chain.is_empty());
    }
}
