use axum::{
    error_handling::HandleErrorLayer,
    http::{
        header::{HeaderValue, STRICT_TRANSPORT_SECURITY},
        StatusCode,
    },
    routing::{get, post},
    BoxError, Router,
};

use config::{AppConfig, Envioronment};
use std::time::Duration;
use tower::ServiceBuilder;
use tower::{buffer::BufferLayer, limit::rate::RateLimitLayer, timeout::TimeoutLayer};
use tower_cookies::CookieManagerLayer;
use tower_http::{
    classify::ServerErrorsFailureClass,
    set_header::SetResponseHeaderLayer,
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::warn;


mod login;
mod middleware;
mod protected_hello;
mod tracing_setup;
mod config;
mod connection;
mod helper;
mod hello;
mod schema;
mod models;
mod signup;

pub async fn create_routes() -> Router {
    let app_config = AppConfig::instance();
    let on_response = DefaultOnResponse::new()
        .level(tracing::Level::INFO)
        .latency_unit(LatencyUnit::Micros);

    let public_routes = Router::new().route("/hello", get(hello::hello))
        .route("/signup",post(signup::signup_handler))
        .route("/login", post(login::loginhandler));

    let protected_routes = Router::new()
        .route("/hello_protected", get(protected_hello::hello))
        .layer(axum::middleware::from_fn(middleware::auth_middleware_with_session));

    
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CookieManagerLayer::new())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(
                    app_config.features.rate_limiting.unwrap_or(100),
                    Duration::from_secs(app_config.features.duration),
                ))
                .layer(TimeoutLayer::new(Duration::from_secs(
                    app_config.features.timeout_seconds,
                )))
                .layer(SetResponseHeaderLayer::if_not_present(
                    STRICT_TRANSPORT_SECURITY,
                    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
                ))
                .layer(
                    TraceLayer::new_for_http()
                        .on_response(on_response)
                        .on_failure(
                            |error: ServerErrorsFailureClass, _latency: Duration, _span: &tracing::Span| {
                                if let ServerErrorsFailureClass::StatusCode(code) = error {
                                    if code == StatusCode::TOO_MANY_REQUESTS {
                                        warn!("Rate limit exceeded: {}", code);
                                    }
                                }
                            },
                        ),
                ),
        )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Envioronment::from_env() {
        Envioronment::Development => dotenv::from_filename(".env.local").ok(),
        Envioronment::Production => dotenv::from_filename(".env.production").ok(),
        _ => dotenv::from_filename(".env.local").ok(),
    };

    tracing_setup::setup_console_tracing();
    let app_config = AppConfig::instance();

    // Compose app with all layers
    let app = create_routes().await;

    println!("Starting server with configuration: {:?}", app_config);
    let addr = format!("{}:{}", app_config.server.host, app_config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
