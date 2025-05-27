mod app;
mod auth;
mod db;
mod web;

use crate::app::App;
use crate::auth::backend::Backend;
use crate::db::types::PooledConnection;
use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{middleware, Router};
use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::tower_sessions::{Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use diesel_async_migrations::embed_migrations;
use std::env;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::Key;
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = App::new().await;

    {
        let mut conn: PooledConnection = app.db.get().await?;
        embed_migrations!("migrations/")
            .run_pending_migrations(&mut conn)
            .await?;
    }

    let pool = Pool::new(Config::default(), None, None, None, 6)?;
    pool.connect();
    pool.wait_for_connect().await?;

    let key = Key::from(env::var("SECRET_KEY")?.as_bytes());
    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)))
        .with_signed(key);

    let backend = Backend::new(app.db.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let static_folder = "frontend/out";
    let not_found_page = ServeFile::new(format!("{static_folder}/404/index.html"));
    let frontend_service = ServiceBuilder::new()
        .layer(middleware::from_fn(set_static_cache_control))
        .service(ServeDir::new(static_folder).not_found_service(not_found_page));

    let router = Router::new()
        .merge(web::router())
        .fallback_service(frontend_service)
        .layer(TraceLayer::new_for_http())
        .layer(auth_layer)
        .with_state(app);

    let ip = env::var("APP_IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let address = format!("{ip}:{port}");

    info!("Listening on {address}");

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn set_static_cache_control(request: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    if cfg!(not(debug_assertions)) {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        );
    }
    response
}
