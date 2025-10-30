use base64::{engine::general_purpose, Engine as _};
use rsa::{pkcs1::EncodeRsaPrivateKey, pkcs8::DecodePrivateKey, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: uuid::Uuid,
    pub user_uuid: uuid::Uuid,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_uuid: String,
    pub token_uuid: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaimsProcess {
    pub user_uuid: String,
}

pub fn generate_jwt_token(
    user_uuid: uuid::Uuid,
    ttl: i64,
    private_key: String,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    // Determine format: PEM ascii vs base64-encoded PEM and decode with padding
    let pem_bytes = if private_key.contains("-----BEGIN") {
        tracing::info!(
            "access private key: detected PEM ascii, len={}",
            private_key.len()
        );
        private_key.into_bytes()
    } else {
        let mut b64 = private_key.replace('\r', "").replace('\n', "");
        let pad = b64.len() % 4;
        if pad != 0 {
            let to_add = 4 - pad;
            tracing::warn!(
                "access private key: base64 missing padding, adding {} '='",
                to_add
            );
            b64.push_str(&"=".repeat(to_add));
        }
        match general_purpose::STANDARD.decode(b64.as_bytes()) {
            Ok(bytes) => {
                tracing::info!(
                    "access private key: base64 decode ok, bytes={}",
                    bytes.len()
                );
                bytes
            }
            Err(e) => {
                tracing::error!("access private key: base64 decode error: {:?}", e);
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                ));
            }
        }
    };

    let now = chrono::Utc::now();
    let mut token_details = TokenDetails {
        user_uuid,
        token_uuid: Uuid::new_v4(),
        expires_in: Some((now + chrono::Duration::minutes(ttl)).timestamp()),
        token: None,
    };

    let claims = TokenClaims {
        user_uuid: token_details.user_uuid.to_string(),
        token_uuid: token_details.token_uuid.to_string(),
        exp: token_details.expires_in.unwrap(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
    };

    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

    // Try RSA PEM first; if it fails, convert PKCS#8 to PKCS#1 DER and use RSA DER
    let enc_key = match jsonwebtoken::EncodingKey::from_rsa_pem(&pem_bytes) {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!(
                "EncodingKey::from_rsa_pem failed: {:?}; attempting PKCS#8 -> PKCS#1 conversion",
                e
            );
            match String::from_utf8(pem_bytes.clone()) {
                Ok(pem_str) => {
                    let begin = "-----BEGIN PRIVATE KEY-----";
                    let end = "-----END PRIVATE KEY-----";
                    if let (Some(start_idx), Some(end_idx)) =
                        (pem_str.find(begin), pem_str.find(end))
                    {
                        let inner = &pem_str[start_idx + begin.len()..end_idx];
                        let sanitized: String = inner
                            .chars()
                            .filter(|c| {
                                c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '='
                            })
                            .collect();
                        let removed = inner.len() - sanitized.len();
                        if removed > 0 {
                            tracing::warn!("pkcs8 inner base64: removed {} invalid chars", removed);
                        }
                        let mut b64 = sanitized.replace('\r', "").replace('\n', "");
                        let rem = b64.len() % 4;
                        if rem != 0 {
                            let pad = 4 - rem;
                            b64.extend(std::iter::repeat('=').take(pad));
                            tracing::warn!("pkcs8 inner base64: added {} padding '='", pad);
                        }

                        let engine = base64::engine::general_purpose::GeneralPurpose::new(
                            &base64::alphabet::STANDARD,
                            base64::engine::GeneralPurposeConfig::new()
                                .with_decode_allow_trailing_bits(true),
                        );
                        match engine.decode(b64.as_bytes()) {
                            Ok(der) => {
                                tracing::info!("pkcs8->der decode ok, bytes={}", der.len());
                                match RsaPrivateKey::from_pkcs8_der(&der) {
                                    Ok(rsa_key) => match rsa_key.to_pkcs1_der() {
                                        Ok(der_doc) => {
                                            tracing::info!(
                                                "pkcs1 DER conversion ok, bytes={}",
                                                der_doc.as_bytes().len()
                                            );
                                            jsonwebtoken::EncodingKey::from_rsa_der(
                                                der_doc.as_bytes(),
                                            )
                                        }
                                        Err(err) => {
                                            tracing::error!(
                                                "to_pkcs1_der conversion error: {:?}",
                                                err
                                            );
                                            return Err(jsonwebtoken::errors::Error::from(
                                                jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                                            ));
                                        }
                                    },
                                    Err(err) => {
                                        tracing::error!("from_pkcs8_der parse error: {:?}", err);
                                        return Err(jsonwebtoken::errors::Error::from(
                                            jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                                        ));
                                    }
                                }
                            }
                            Err(err) => {
                                tracing::error!("pkcs8 inner base64 decode error: {:?}", err);
                                return Err(jsonwebtoken::errors::Error::from(
                                    jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                                ));
                            }
                        }
                    } else {
                        tracing::error!("PEM missing PRIVATE KEY header/footer");
                        return Err(jsonwebtoken::errors::Error::from(
                            jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                        ));
                    }
                }
                Err(err) => {
                    tracing::error!("PEM utf8 decode error: {:?}", err);
                    return Err(jsonwebtoken::errors::Error::from(
                        jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                    ));
                }
            }
        }
    };

    let token = jsonwebtoken::encode(&header, &claims, &enc_key)?;
    token_details.token = Some(token);
    Ok(token_details)
}

pub fn verify_jwt_token(
    public_key: String,
    token: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    // Determine format: PEM ascii vs base64-encoded PEM and decode with padding
    let pem_bytes = if public_key.contains("-----BEGIN") {
        tracing::info!(
            "access public key: detected PEM ascii, len={}",
            public_key.len()
        );
        public_key.into_bytes()
    } else {
        let mut b64 = public_key.replace('\r', "").replace('\n', "");
        let pad = b64.len() % 4;
        if pad != 0 {
            let to_add = 4 - pad;
            tracing::warn!(
                "access public key: base64 missing padding, adding {} '='",
                to_add
            );
            b64.push_str(&"=".repeat(to_add));
        }
        match general_purpose::STANDARD.decode(b64.as_bytes()) {
            Ok(bytes) => {
                tracing::info!("access public key: base64 decode ok, bytes={}", bytes.len());
                bytes
            }
            Err(e) => {
                tracing::error!("access public key: base64 decode error: {:?}", e);
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                ));
            }
        }
    };

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(&pem_bytes)?,
        &validation,
    )?;

    let user_uuid = Uuid::parse_str(decoded.claims.user_uuid.as_str()).map_err(|_| {
        jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
    })?;
    let token_uuid = Uuid::parse_str(decoded.claims.token_uuid.as_str()).map_err(|_| {
        jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
    })?;

    Ok(TokenDetails {
        token: None,
        token_uuid,
        user_uuid,
        expires_in: None,
    })
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::config::config::Config;
    use base64::{engine::general_purpose, Engine as _};

    // Ensure minimal env vars for Config::init() during tests
    fn ensure_env() {
        let _ = dotenvy::dotenv();
        std::env::set_var("APP_ENV", "dev");
        std::env::set_var("APP_PORT", "4000");
        std::env::set_var(
            "DATABASE_URL",
            "postgresql://postgres:postgres@127.0.0.1:5432/postgres?schema=public",
        );
        std::env::set_var("REDIS_URL", "redis://127.0.0.1:6379/0");
        std::env::set_var("CLIENT_ORIGIN", "http://localhost:3000");
        std::env::set_var("ACCESS_TOKEN_EXPIRED_IN", "15m");
        std::env::set_var("ACCESS_TOKEN_MAXAGE", "15");
        std::env::set_var("REFRESH_TOKEN_EXPIRED_IN", "60m");
        std::env::set_var("REFRESH_TOKEN_MAXAGE", "60");
    }

    #[test]
    fn generate_and_verify_access_ok() {
        ensure_env();
        let cfg = Config::init();
        let user_uuid = Uuid::new_v4();
        let details = generate_jwt_token(user_uuid, 1, cfg.access_token_private_key.clone())
            .expect("generate access token");
        let token = details.token.as_ref().expect("token present");
        let verified = verify_jwt_token(cfg.access_token_public_key.clone(), token)
            .expect("verify access token");
        assert_eq!(verified.user_uuid, user_uuid);
        assert_eq!(
            verified.token_uuid.to_string(),
            details.token_uuid.to_string()
        );
    }

    #[test]
    fn generate_and_verify_refresh_ok() {
        ensure_env();
        let cfg = Config::init();
        let user_uuid = Uuid::new_v4();
        let details = generate_jwt_token(user_uuid, 1, cfg.refresh_token_private_key.clone())
            .expect("generate refresh token");
        let token = details.token.as_ref().expect("token present");
        let verified = verify_jwt_token(cfg.refresh_token_public_key.clone(), token)
            .expect("verify refresh token");
        assert_eq!(verified.user_uuid, user_uuid);
        assert_eq!(
            verified.token_uuid.to_string(),
            details.token_uuid.to_string()
        );
    }

    #[test]
    fn generate_with_base64_private_ok() {
        ensure_env();
        let cfg = Config::init();
        let user_uuid = Uuid::new_v4();
        let pem_b64 = general_purpose::STANDARD.encode(cfg.access_token_private_key.as_bytes());
        let details = generate_jwt_token(user_uuid, 1, pem_b64).expect("generate with base64");
        let token = details.token.as_ref().expect("token present");
        let verified = verify_jwt_token(cfg.access_token_public_key.clone(), token)
            .expect("verify token using ascii public key");
        assert_eq!(verified.user_uuid, user_uuid);
    }
}
