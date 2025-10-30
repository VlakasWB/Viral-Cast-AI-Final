mod config {
    pub mod config;
}

mod middleware {
    pub mod jwt;
    pub mod logging;
}

mod models {
    pub mod categories;
    pub mod ingredient_catalog;
    pub mod profiles;
    pub mod roles;
    pub mod units_of_measure;
    pub mod user;
    // pub mod stock_moves; // removed
    pub mod ai_config;
    pub mod orders;
    pub mod payments;
    pub mod products;
    pub mod rag;
    pub mod recipe_items;
    pub mod recipe_sets;
    // pub mod sales_daily; // removed
    pub mod forecast_daily;
    pub mod ingredient_market_prices;
    pub mod ingredient_stock_moves;
    pub mod ingredient_stocks;

    pub mod regions;
    pub mod store_ingredient_predictions;
    pub mod store_product_predictions;
    pub mod stores;
    pub mod trend_news;
    pub mod weather_bmkg;
}
mod data {
    pub mod master {
        pub mod master;
        pub mod roles;
    }
}
mod dto {
    pub mod api;
    pub mod auth;
    pub mod auth_token;
    pub mod categories;
    pub mod ingredient_catalog;
    pub mod profiles;
    pub mod roles;
    pub mod units_of_measure;
    pub mod users;
    // pub mod stock_moves; // removed
    pub mod ai;
    pub mod orders;
    pub mod payments;
    pub mod products;
    pub mod rag;
    pub mod recipe_items;
    pub mod recipe_sets;
    // pub mod sales_daily; // removed
    pub mod forecast_daily;
    pub mod ingredient_market_prices;
    pub mod ingredient_stock_moves;
    pub mod ingredient_stocks;

    pub mod google_ads;
    pub mod regions;
    pub mod store_ingredient_predictions;
    pub mod stores;
    pub mod weather_bmkg;
    // Added i18n DTO module
    pub mod i18n;
    pub mod store_product_predictions;
    pub mod trend_news;
}
mod handlers {
    pub mod auth;
    pub mod categories;
    pub mod ingredient_catalog;
    pub mod profiles;
    pub mod roles;
    pub mod units_of_measure;
    // pub mod stock_moves; // removed
    pub mod ai;
    pub mod images;
    pub mod orders;
    pub mod payments;
    pub mod products;
    pub mod rag;
    pub mod recipe_items;
    pub mod recipe_sets;
    // pub mod sales_daily; // removed
    pub mod forecast_daily;
    pub mod ingredient_market_prices;
    pub mod ingredient_stock_moves;
    pub mod ingredient_stocks;

    pub mod google_ads;
    pub mod regions;
    pub mod store_ingredient_predictions;
    pub mod stores;
    pub mod weather_bmkg;
    // Added i18n handlers module
    pub mod i18n;
    pub mod store_product_predictions;
    pub mod trend_news;
}

mod routes {
    pub mod auth;
    pub mod categories;
    pub mod ingredient_catalog;
    pub mod profiles;
    pub mod roles;
    pub mod units_of_measure;
    // pub mod stock_moves; // removed
    pub mod ai;
    pub mod images;
    pub mod orders;
    pub mod payments;
    pub mod products;
    pub mod rag;
    pub mod recipe_items;
    pub mod recipe_sets;
    // pub mod sales_daily; // removed
    pub mod forecast_daily;
    pub mod ingredient_market_prices;
    pub mod ingredient_stock_moves;
    pub mod ingredient_stocks;

    pub mod google_ads;
    pub mod regions;
    pub mod stores;
    pub mod weather_bmkg;
    // Added i18n routes module
    pub mod i18n;
    pub mod trend_news;
}

mod repository {
    pub mod ai;
    pub mod auth;
    pub mod categories;
    pub mod ingredient_catalog;
    pub mod products;
    pub mod recipe_items;
    pub mod recipe_sets;
    pub mod regions;
    pub mod roles;
    pub mod stores;
    pub mod units_of_measure;
    pub mod weather_bmkg;
    // Add missing repository modules for ingredients-related features
    pub mod ingredient_market_prices;
    pub mod ingredient_stock_moves;
    pub mod ingredient_stocks;
    // Added i18n repository module
    pub mod i18n;
    pub mod store_ingredient_predictions;
    pub mod store_product_predictions;
    pub mod trend_news;
}

mod services {
    pub mod google_ads;
    pub mod milvus;
    pub mod trend_news;
    pub mod xendit;
    // ID: Tambahkan modul services baru untuk rate limiting, batching, dan scheduler
    // EN: Add new services modules for rate limiting, batching, and scheduler
    pub mod batch_processor;
    pub mod job_scheduler;
    pub mod rate_limiter;
    // ID: Nonaktifkan modul yang belum siap untuk produksi agar kompilasi sukses
    // EN: Disable not-ready modules to keep compilation successful
    // ID: Aktifkan kembali modul layanan untuk kompilasi penuh.
    // EN: Re-enable service modules for full compilation.
    // ID: Nonaktifkan modul layanan agar build stabil sementara.
    // EN: Disable service modules to stabilize the build for now.
    // pub mod ingredient_scheduler;
    // pub mod ingredient_prediction_service;
}

mod workers {
    pub mod bmkg_scheduler;
}

use config::config::Config;
use std::sync::Arc;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderName, HeaderValue, Method,
};
use axum::Router;
use dotenvy::dotenv;
use redis::Client;
use routes::auth::create_auth_router;
use routes::categories::create_categories_router;
use routes::ingredient_catalog::create_ingredients_router;
use routes::orders::create_orders_router;
use routes::payments::create_payments_router;
use routes::products::create_products_router;
use routes::profiles::create_profiles_router;
use routes::recipe_items::create_recipe_items_router;
use routes::recipe_sets::create_recipe_sets_router;
use routes::roles::create_roles_router;
use std::fs;
// use routes::stock_moves::create_stock_moves_router; // removed
// use routes::uom_conversions::create_uom_conversions_router; // removed
use crate::data::master::master::master_roles;
use routes::ai::create_ai_router;
use routes::images::create_images_router;
use routes::rag::create_rag_router;
use routes::units_of_measure::create_units_of_measure_router;
// use routes::sales_daily::create_sales_daily_router; // removed
use routes::forecast_daily::create_forecast_daily_router;

use milvus::{Client as MilvusClient, Endpoint};
use routes::google_ads::create_google_ads_router;
use routes::ingredient_market_prices::create_ingredient_market_prices_router;
use routes::ingredient_stock_moves::create_ingredient_stock_moves_router;
use routes::ingredient_stocks::create_ingredient_stocks_router;
use routes::regions::create_regions_routes;
use routes::stores::create_stores_router;
use routes::trend_news::create_trend_news_router;
use routes::weather_bmkg::weather_bmkg_routes;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Pool, Postgres,
};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_appender::rolling;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// New import for i18n router
use routes::i18n::create_i18n_router;

#[derive(Clone)]
pub struct AppState {
    db: Pool<Postgres>,
    env: Config,
    redis_client: Client,
    // In-memory session store fallback when Redis is unavailable
    session_store: Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
    // BMKG toggles derived from env
    bmkg_db_first: bool,
    bmkg_save_json: bool,
    // Milvus
    milvus_client: Option<Arc<tokio::sync::Mutex<MilvusClient>>>,
    milvus_collection: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize tracing subscriber for better debugging
    // Ensure logs directory exists
    let _ = fs::create_dir_all("logs");

    // Configure daily rolling file appender
    let file_appender = rolling::daily("logs", "vcai.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info,axum::rejection=trace".into()),
        )
        // Console logs
        .with(tracing_subscriber::fmt::layer())
        // File logs
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    let config = Config::init();

    let pool = if config.allow_mock_dependencies {
        println!("⚠️ Running with mock dependencies; database connectivity will be checked lazily");
        let options = config
            .database_url
            .parse::<PgConnectOptions>()
            .unwrap_or_else(|err| {
                println!(
                    "🔥 Failed to parse DATABASE_URL for lazy connection: {:?}",
                    err
                );
                std::process::exit(1);
            });
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy_with(options)
    } else {
        match PgPoolOptions::new()
            .max_connections(config.db_max_connections.unwrap_or(20))
            .acquire_timeout(Duration::from_secs(
                config.db_acquire_timeout_secs.unwrap_or(30),
            ))
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => {
                println!("✅Connection to the database is successful!");
                pool
            }
            Err(err) => {
                println!("🔥 Failed to connect to the database: {:?}", err);
                std::process::exit(1);
            }
        }
    };

    match master_roles(&pool).await {
        Ok(_) => println!("✅ [startup] Master roles seed check completed"),
        Err(err) => println!("🔥 [startup] Failed to ensure master roles: {:?}", err),
    }
    let redis_client = match Client::open(config.redis_url.to_owned()) {
        Ok(client) => {
            println!("✅Connection to the redis is successful!");
            client
        }
        Err(e) => {
            println!("🔥 Error connecting to Redis: {}", e);
            std::process::exit(1);
        }
    };

    // Create list of allowed origins
    let mut allowed_origins = vec![config.client_origin.clone()];

    // Add localhost ports 5171-5180
    for port in 5171..=5180 {
        allowed_origins.push(format!("http://localhost:{}", port));
    }

    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_credentials(true)
        .allow_headers([
            AUTHORIZATION,
            ACCEPT,
            CONTENT_TYPE,
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("x-csrf-token"),
        ]);

    let app_state = Arc::new(AppState {
        db: pool.clone(),
        env: config.clone(),
        redis_client: redis_client.clone(),
        session_store: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        bmkg_db_first: config.bmkg_db_first,
        bmkg_save_json: config.bmkg_save_json,
        milvus_client: {
            if let Some(uri) = config.milvus_uri.clone() {
                match Endpoint::from_shared(uri.clone()) {
                    Ok(ep) => match MilvusClient::connect("default".to_string(), ep).await {
                        Ok(client) => Some(Arc::new(tokio::sync::Mutex::new(client))),
                        Err(e) => {
                            tracing::warn!("Failed to connect Milvus: {:?}", e);
                            None
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Invalid Milvus endpoint: {:?}", e);
                        None
                    }
                }
            } else {
                None
            }
        },
        milvus_collection: config.milvus_collection.clone(),
    });

    if config.allow_mock_dependencies {
        println!("⚠️ [startup] Skipping Milvus collection checks (mock dependency mode)");
    } else if let Some(ref client) = app_state.milvus_client {
        // Read embedding dimension from configuration table, fallback to 1536
        let _embedding_dim = match sqlx::query_scalar::<_, i32>(
            "SELECT embedding_dimensions FROM rag_configurations WHERE deleted_at = 0 ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&app_state.db)
        .await {
            Ok(Some(dim)) => dim,
            Ok(None) => 1536,
            Err(e) => {
                println!("⚠️ Failed to read embedding dimension from DB: {:?}", e);
                1536
            }
        };

        let mut guard = client.lock().await;
        if let Err(e) =
            services::milvus::ensure_rag_collection(&mut guard, &app_state.milvus_collection).await
        {
            println!(
                "⚠️ Failed to ensure Milvus collection '{}': {:?}",
                &app_state.milvus_collection, e
            );
        }
    }

    let auth_router = create_auth_router(app_state.clone());
    let categories_router = create_categories_router(app_state.clone());
    let ingredients_router = create_ingredients_router(app_state.clone());
    let orders_router = create_orders_router(app_state.clone());
    let payments_router = create_payments_router(app_state.clone());
    let products_router = create_products_router(app_state.clone());
    let profiles_router = create_profiles_router(app_state.clone());
    let stores_router = create_stores_router(app_state.clone());
    let ingredient_market_prices_router = create_ingredient_market_prices_router(app_state.clone());
    let ingredient_stock_moves_router = create_ingredient_stock_moves_router(app_state.clone());
    let ingredient_stocks_router = create_ingredient_stocks_router(app_state.clone());
    let recipe_items_router = create_recipe_items_router(app_state.clone());
    let recipe_sets_router = create_recipe_sets_router(app_state.clone());
    let roles_router = create_roles_router(app_state.clone());
    // let stock_moves_router = create_stock_moves_router(app_state.clone()); // removed
    // let uom_conversions_router = create_uom_conversions_router(app_state.clone()); // removed
    let uoms_router = create_units_of_measure_router(app_state.clone());
    let images_router = create_images_router(app_state.clone());
    let ai_router = create_ai_router(app_state.clone());
    let rag_router = create_rag_router();
    // let sales_daily_router = create_sales_daily_router(app_state.clone()); // removed
    let forecast_daily_router = create_forecast_daily_router(app_state.clone());

    let weather_bmkg_router = weather_bmkg_routes();
    let regions_router = create_regions_routes();
    let google_ads_router = create_google_ads_router(app_state.clone());
    // New: create i18n router
    let i18n_router = create_i18n_router(app_state.clone());
    let trend_news_router = create_trend_news_router(app_state.clone());

    let app = Router::new()
        .nest("/", auth_router)
        .nest("/", categories_router)
        .nest("/", ingredients_router)
        .nest("/", orders_router)
        .nest("/", payments_router)
        .nest("/", products_router)
        .nest("/", profiles_router)
        .nest("/", stores_router)
        .nest("/", ingredient_market_prices_router)
        .nest("/", ingredient_stock_moves_router)
        .nest("/", recipe_items_router)
        .nest("/", recipe_sets_router)
        .nest("/", roles_router)
        // .nest("/", stock_moves_router) // removed
        // .nest("/", uom_conversions_router) // removed
        .nest("/", uoms_router)
        .nest("/", images_router)
        .nest("/", ai_router)
        .nest("/", ingredient_stocks_router)
        .nest("/api/rag", rag_router.with_state(app_state.clone()))
        // .nest("/api/sales-daily", sales_daily_router) // removed
        .nest("/api/forecast-daily", forecast_daily_router)
        .nest("/", weather_bmkg_router.with_state((*app_state).clone()))
        .nest("/", regions_router.with_state((*app_state).clone()))
        .nest("/", google_ads_router)
        // New: nest i18n routes
        .nest("/", i18n_router)
        .nest("/", trend_news_router)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
        .layer(axum::middleware::from_fn(
            crate::middleware::logging::api_logger,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.app_port);
    println!(
        "🚀 Server started successfully at http://localhost:{}",
        config.app_port
    );
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Start BMKG scheduler in background (50 hits/hour ~ every 72 seconds)
    if config.bmkg_scheduler_enabled {
        let scheduler_state = app_state.clone();
        tokio::spawn(async move {
            workers::bmkg_scheduler::start_bmkg_scheduler(scheduler_state).await;
        });

        // Start BMKG Redis queue workers when enabled
        if config.bmkg_use_queue {
            let worker_state = app_state.clone();
            let concurrency = config.bmkg_queue_workers;
            tokio::spawn(async move {
                workers::bmkg_scheduler::start_bmkg_queue_workers(worker_state, concurrency).await;
            });
            println!(
                "🧵 BMKG queue workers started with concurrency {} (Redis)",
                concurrency
            );
        } else {
            println!("⚠️ BMKG Redis queue disabled by config; scheduler runs inline/fallback");
        }
    } else {
        println!("⚠️ BMKG scheduler disabled by config");
    }

    axum::serve(listener, app).await.unwrap()
}
