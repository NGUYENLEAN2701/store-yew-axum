mod config;
mod db;
mod error;
mod models;
mod routes;
mod security;
mod seed;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use config::Config;
use security::rate_limit_captcha::CaptchaState;

#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Database,
    pub config: Arc<Config>,
    pub captcha: CaptchaState,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();
    let db = db::connect(&config).await;
    seed::seed_products(&db).await;

    let state = AppState {
        db,
        config: Arc::new(config),
        captcha: CaptchaState::new(),
    };

    let api_router = Router::new()
        .route("/products", get(routes::products::list_products))
        .route("/products/:id", get(routes::products::get_product))
        .route("/orders", post(routes::orders::create_order))
        .route("/setup/status", get(routes::auth::setup_status))
        .route("/admin/setup", post(routes::auth::setup_admin))
        .route("/admin/login", post(routes::auth::login))
        .route("/admin/logout", post(routes::auth::logout))
        .route("/admin/me", get(routes::auth::me))
        .route("/captcha/challenge", get(routes::captcha::challenge))
        .route("/captcha/verify", post(routes::captcha::verify))
        .route("/admin/products", post(routes::products::create_product))
        .route(
            "/admin/products/:id",
            put(routes::products::update_product).delete(routes::products::delete_product),
        )
        .route("/admin/orders", get(routes::orders::list_orders))
        .route(
            "/admin/orders/:id/status",
            put(routes::orders::update_order_status),
        )
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security::rate_limit_captcha::rate_limit_captcha,
        ))
        .with_state(state.clone());

    let dist_dir = std::env::var("DIST_DIR").unwrap_or_else(|_| "dist".to_string());
    let index_path = format!("{dist_dir}/index.html");
    let static_service = ServeDir::new(&dist_dir).fallback(ServeFile::new(index_path));

    let port = state.config.port;
    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(static_service)
        .layer(middleware::from_fn(security::headers::security_headers))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("GreenIEM backend listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
