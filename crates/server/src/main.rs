mod auth;
mod db;
mod middleware;
mod offline_sweep;
mod routes;
mod session;
mod state;
mod system_auth;

use axum::{
    http::Method,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::sync::Arc;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://shawk:shawk@localhost:5432/shawk".to_string());
    let admin_token = std::env::var("ADMIN_TOKEN").unwrap_or_default();
    if admin_token.is_empty() {
        tracing::warn!("ADMIN_TOKEN is not set - probe registration endpoint is disabled");
    }
    let required_group = std::env::var("DASHBOARD_GROUP").unwrap_or_else(|_| "syshawk".to_string());
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("migrations applied");

    let state = Arc::new(AppState::new(pool, admin_token, required_group.clone()));
    info!(group = %required_group, "dashboard access restricted to this host group");

    tokio::spawn(offline_sweep::run(state.clone()));

    // Routes with their own auth (probe bearer tokens, or the admin
    // shared secret) - never gated by the dashboard session cookie.
    let public = Router::new()
        .route("/api/ingest", post(routes::ingest::ingest))
        .route("/api/probes", post(routes::probes::register))
        .route("/api/probe/control", get(routes::terminal::probe_control))
        .route(
            "/api/probe/terminal-connect",
            get(routes::terminal::probe_terminal_connect),
        )
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/auth/me", get(routes::auth::me))
        .route("/healthz", get(|| async { "ok" }));

    // Routes a logged-in dashboard user can reach.
    let protected = Router::new()
        .route("/api/probes", get(routes::probes::list))
        .route("/api/probes/:id/history", get(routes::probes::history))
        .route("/api/probes/:id/terminal", get(routes::terminal::dashboard_terminal))
        .route("/api/ws", get(routes::ws::ws_handler))
        .route_layer(from_fn_with_state(state.clone(), middleware::require_session));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!(addr = %bind_addr, "shawk-server listening");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
