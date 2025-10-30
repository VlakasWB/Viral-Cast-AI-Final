fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set", var_name))
}

// Prefer reading key material from file (default path first), then explicit path, then env
fn get_key_or_path(var_name: &str, path_var_name: &str, default_path: &str) -> String {
    use std::path::Path;

    // 1) Default path under repo if readable
    if let Ok(contents) = std::fs::read_to_string(default_path) {
        if !contents.trim().is_empty() {
            return contents;
        }
    }
    // 2) Explicit file path via env
    if let Ok(path) = std::env::var(path_var_name) {
        if !path.is_empty() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if !contents.trim().is_empty() {
                    return contents;
                }
            }
        }
    }
    // 3) Raw env value
    if let Ok(val) = std::env::var(var_name) {
        if !val.trim().is_empty() {
            return val;
        }
    }

    // Ensure default directory exists so downstream code can generate keys lazily
    if let Some(parent) = Path::new(default_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Returning an empty string allows ensure_valid_rsa_pems to generate keys automatically
    String::new()
}

fn get_optional_secret(var_name: &str, file_var_names: &[&str]) -> Option<String> {
    // Prefer explicit file paths to avoid exposing secrets via process list
    for file_var in file_var_names {
        if let Ok(path) = std::env::var(file_var) {
            let trimmed_path = path.trim();
            if trimmed_path.is_empty() {
                continue;
            }
            if let Ok(contents) = std::fs::read_to_string(trimmed_path) {
                let trimmed = contents.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    if let Ok(value) = std::env::var(var_name) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

// Ensure RSA keys are valid; if not, generate fresh 2048-bit RSA PEMs (PKCS#8 private, SPKI public)
fn ensure_valid_rsa_pems(
    access_priv: String,
    access_pub: String,
    refresh_priv: String,
    refresh_pub: String,
) -> (String, String, String, String) {
    let priv_ok = jsonwebtoken::EncodingKey::from_rsa_pem(access_priv.as_bytes()).is_ok();
    let pub_ok = jsonwebtoken::DecodingKey::from_rsa_pem(access_pub.as_bytes()).is_ok();
    let r_priv_ok = jsonwebtoken::EncodingKey::from_rsa_pem(refresh_priv.as_bytes()).is_ok();
    let r_pub_ok = jsonwebtoken::DecodingKey::from_rsa_pem(refresh_pub.as_bytes()).is_ok();

    if priv_ok && pub_ok && r_priv_ok && r_pub_ok {
        return (access_priv, access_pub, refresh_priv, refresh_pub);
    }

    // Generate new RSA key pair
    let mut rng = rand::rngs::OsRng;
    let rsa_key = rsa::RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate RSA key");

    use rsa::pkcs8::EncodePrivateKey as _;
    use rsa::pkcs8::EncodePublicKey as _;
    use rsa::pkcs8::LineEnding;
    let priv_pem = rsa_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("failed to encode pkcs8 private pem")
        .to_string();
    let pub_pem = rsa::RsaPublicKey::from(&rsa_key)
        .to_public_key_pem(LineEnding::LF)
        .expect("failed to encode spki public pem")
        .to_string();

    // Best-effort write to tmp paths used by config defaults
    let _ = std::fs::create_dir_all("tmp");
    let _ = std::fs::write("tmp/access_priv.pem", &priv_pem);
    let _ = std::fs::write("tmp/access_pub.pem", &pub_pem);
    let _ = std::fs::write("tmp/refresh_priv.pem", &priv_pem);

    (priv_pem.clone(), pub_pem.clone(), priv_pem, pub_pem)
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub app_port: String,

    pub database_url: String,
    pub redis_url: String,
    pub client_origin: String,
    // Optional DB pool tuning
    pub db_max_connections: Option<u32>,
    pub db_acquire_timeout_secs: Option<u64>,

    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,

    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_expires_in: String,
    pub refresh_token_max_age: i64,

    // BMKG feature toggles
    // Enable reading BMKG predictions from DB first (Y/N)
    pub bmkg_db_first: bool,
    // Enable saving fetched BMKG JSON to disk under data/json (Y/N)
    pub bmkg_save_json: bool,
    // Enable background BMKG scheduler loop (default true)
    pub bmkg_scheduler_enabled: bool,
    // Enable Redis-backed BMKG queue (default true)
    pub bmkg_use_queue: bool,
    // Number of BMKG queue workers (default 2)
    pub bmkg_queue_workers: usize,

    // Milvus & Embedding config
    pub milvus_uri: Option<String>,
    pub milvus_token: Option<String>,
    pub milvus_collection: String,
    pub openai_api_key: Option<String>,
    pub allow_mock_dependencies: bool,
    pub serper_api_key: Option<String>,
    pub serper_base_url: Option<String>,
    pub serper_default_gl: Option<String>,
    pub serper_default_hl: Option<String>,
    // Xendit QRIS config
    pub xendit_secret_key_sandbox: Option<String>,
    pub xendit_secret_key_live: Option<String>,
    pub xendit_callback_token_sandbox: Option<String>,
    pub xendit_callback_token_live: Option<String>,
    pub xendit_qris_callback_url: Option<String>,
    // Google Ads API config
    pub google_ads_client_id: Option<String>,
    pub google_ads_client_secret: Option<String>,
    pub google_ads_refresh_token: Option<String>,
    pub google_ads_developer_token: Option<String>,
    pub google_ads_login_customer_id: Option<String>,
    pub google_ads_customer_id: Option<String>,
}

impl Config {
    pub fn init() -> Config {
        // ID: Coba muat `.env-ai` terlebih dahulu sesuai standar pengguna.
        // EN: Try loading `.env-ai` first as per user standard.
        let using_ai_env = dotenvy::from_filename(".env-ai").is_ok();
        if !using_ai_env {
            // ID: Jika `.env-ai` tidak ada, fallback ke `.env`.
            // EN: If `.env-ai` is not present, fallback to `.env`.
            let _ = dotenvy::dotenv();
        }

        // ID: APP_ENV default ke "AI" saat menggunakan .env-ai agar server tidak gagal start.
        // EN: Default APP_ENV to "AI" when using .env-ai to avoid startup failure.
        let app_env = std::env::var("APP_ENV")
            .unwrap_or_else(|_| {
                if using_ai_env {
                    "AI".to_string()
                } else {
                    panic!("APP_ENV must be set")
                }
            })
            .to_uppercase();
        // ID: Gunakan SERVER_PORT sebagai fallback untuk APP_PORT saat testing .env-ai.
        // EN: Use SERVER_PORT as a fallback for APP_PORT when testing with .env-ai.
        let app_port = std::env::var("APP_PORT")
            .or_else(|_| std::env::var("SERVER_PORT"))
            .unwrap_or_else(|_| {
                if using_ai_env {
                    "8081".to_string()
                } else {
                    panic!("APP_PORT must be set")
                }
            })
            .to_uppercase();

        let database_url = get_env_var("DATABASE_URL");
        let redis_url = get_env_var("REDIS_URL");
        // ID: Default CLIENT_ORIGIN untuk mode AI agar CORS tidak menahan request lokal.
        // EN: Default CLIENT_ORIGIN for AI mode to avoid CORS blocking local requests.
        let client_origin = std::env::var("CLIENT_ORIGIN").unwrap_or_else(|_| {
            if using_ai_env {
                "http://localhost:12000".to_string()
            } else {
                panic!("CLIENT_ORIGIN must be set")
            }
        });
        // Optional DB tuning envs
        let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok());
        let db_acquire_timeout_secs = std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok());

        // Keys: prefer repo files to avoid malformed env encoding
        let access_token_private_key = get_key_or_path(
            "ACCESS_TOKEN_PRIVATE_KEY",
            "ACCESS_TOKEN_PRIVATE_KEY_FILE",
            "tmp/access_priv.pem",
        );
        let access_token_public_key = get_key_or_path(
            "ACCESS_TOKEN_PUBLIC_KEY",
            "ACCESS_TOKEN_PUBLIC_KEY_FILE",
            "tmp/access_pub.pem",
        );
        // ID: Sediakan default aman untuk expiry & max-age token saat AI testing.
        // EN: Provide safe defaults for token expiry & max-age during AI testing.
        let access_token_expires_in =
            std::env::var("ACCESS_TOKEN_EXPIRED_IN").unwrap_or_else(|_| {
                if using_ai_env {
                    "15".to_string()
                } else {
                    panic!("ACCESS_TOKEN_EXPIRED_IN must be set")
                }
            });
        let access_token_max_age = std::env::var("ACCESS_TOKEN_MAXAGE").unwrap_or_else(|_| {
            if using_ai_env {
                "30".to_string()
            } else {
                panic!("ACCESS_TOKEN_MAXAGE must be set")
            }
        });

        let refresh_token_private_key = get_key_or_path(
            "REFRESH_TOKEN_PRIVATE_KEY",
            "REFRESH_TOKEN_PRIVATE_KEY_FILE",
            "tmp/refresh_priv.pem",
        );
        let refresh_token_public_key = get_key_or_path(
            "REFRESH_TOKEN_PUBLIC_KEY",
            "REFRESH_TOKEN_PUBLIC_KEY_FILE",
            "tmp/access_pub.pem",
        );
        let refresh_token_expires_in =
            std::env::var("REFRESH_TOKEN_EXPIRED_IN").unwrap_or_else(|_| {
                if using_ai_env {
                    "15".to_string()
                } else {
                    panic!("REFRESH_TOKEN_EXPIRED_IN must be set")
                }
            });
        let refresh_token_max_age = std::env::var("REFRESH_TOKEN_MAXAGE").unwrap_or_else(|_| {
            if using_ai_env {
                "30".to_string()
            } else {
                panic!("REFRESH_TOKEN_MAXAGE must be set")
            }
        });

        // Validate PEMs; if invalid or empty, generate fresh RSA key pair
        let (
            access_token_private_key,
            access_token_public_key,
            refresh_token_private_key,
            refresh_token_public_key,
        ) = ensure_valid_rsa_pems(
            access_token_private_key,
            access_token_public_key,
            refresh_token_private_key,
            refresh_token_public_key,
        );

        // Optional toggles; default to "N" when not set
        let bmkg_db_first = std::env::var("BMKG_DB_FIRST").unwrap_or_else(|_| "N".to_string());
        let bmkg_save_json = std::env::var("BMKG_SAVE_JSON").unwrap_or_else(|_| "N".to_string());
        let bmkg_scheduler_enabled = std::env::var("BMKG_SCHEDULER_ENABLED")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "y"))
            .unwrap_or(true);
        let bmkg_use_queue = std::env::var("BMKG_USE_QUEUE")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "y"))
            .unwrap_or(true);
        let bmkg_queue_workers = std::env::var("BMKG_QUEUE_WORKERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2);

        // Milvus & Embedding environment
        let milvus_uri = std::env::var("MILVUS_URI").ok();
        let milvus_token = std::env::var("MILVUS_TOKEN").ok();
        let milvus_collection =
            std::env::var("MILVUS_COLLECTION").unwrap_or_else(|_| "rag_chunks".to_string());
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        // ID: Saat AI mode, default izinkan dependency mock agar Milvus optional.
        // EN: In AI mode, default allow mock dependencies so Milvus is optional.
        let allow_mock_dependencies = std::env::var("ALLOW_MOCK_DEPENDENCIES")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(if using_ai_env { true } else { false });
        // ID: SERPER_API_KEY opsional, digunakan untuk sinkronisasi tren F&B via Serper.dev.
        // EN: Optional SERPER_API_KEY used for F&B trend sync via Serper.dev.
        let serper_api_key = get_optional_secret(
            "SERPER_API_KEY",
            &["SERPER_API_KEY_FILE", "SERPER_API_KEY_PATH"],
        );
        let serper_base_url = std::env::var("SERPER_BASE_URL").ok();
        let serper_default_gl = std::env::var("SERPER_DEFAULT_GL").ok();
        let serper_default_hl = std::env::var("SERPER_DEFAULT_HL").ok();

        // Xendit QRIS environment
        let xendit_secret_key_sandbox = std::env::var("XENDIT_SECRET_KEY_SANDBOX").ok();
        let xendit_secret_key_live = std::env::var("XENDIT_SECRET_KEY_LIVE").ok();
        let xendit_callback_token_sandbox = std::env::var("XENDIT_CALLBACK_TOKEN_SANDBOX").ok();
        let xendit_callback_token_live = std::env::var("XENDIT_CALLBACK_TOKEN_LIVE").ok();
        let xendit_qris_callback_url = std::env::var("XENDIT_QRIS_CALLBACK_URL").ok();

        // Google Ads API environment
        let google_ads_client_id = std::env::var("GOOGLE_ADS_CLIENT_ID").ok();
        let google_ads_client_secret = std::env::var("GOOGLE_ADS_CLIENT_SECRET").ok();
        let google_ads_refresh_token = std::env::var("GOOGLE_ADS_REFRESH_TOKEN").ok();
        let google_ads_developer_token = std::env::var("GOOGLE_ADS_DEVELOPER_TOKEN").ok();
        let google_ads_login_customer_id = std::env::var("GOOGLE_ADS_LOGIN_CUSTOMER_ID").ok();
        let google_ads_customer_id = std::env::var("GOOGLE_ADS_CUSTOMER_ID").ok();

        Config {
            app_env,
            app_port,
            database_url,
            redis_url,
            client_origin,
            db_max_connections,
            db_acquire_timeout_secs,
            access_token_private_key,
            access_token_public_key,
            refresh_token_private_key,
            refresh_token_public_key,
            access_token_expires_in,
            refresh_token_expires_in,
            access_token_max_age: access_token_max_age.parse::<i64>().unwrap(),
            refresh_token_max_age: refresh_token_max_age.parse::<i64>().unwrap(),
            bmkg_db_first: bmkg_db_first.eq_ignore_ascii_case("Y"),
            bmkg_save_json: bmkg_save_json.eq_ignore_ascii_case("Y"),
            bmkg_scheduler_enabled,
            bmkg_use_queue,
            bmkg_queue_workers,
            milvus_uri,
            milvus_token,
            milvus_collection,
            openai_api_key,
            allow_mock_dependencies,
            serper_api_key,
            serper_base_url,
            serper_default_gl,
            serper_default_hl,
            xendit_secret_key_sandbox,
            xendit_secret_key_live,
            xendit_callback_token_sandbox,
            xendit_callback_token_live,
            xendit_qris_callback_url,
            google_ads_client_id,
            google_ads_client_secret,
            google_ads_refresh_token,
            google_ads_developer_token,
            google_ads_login_customer_id,
            google_ads_customer_id,
        }
    }
}
