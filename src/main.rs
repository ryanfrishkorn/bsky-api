use axum::{Json, Router, response::IntoResponse, routing::get};
use log::info;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, serde::Serialize)]
struct JsonData {
    unix_ts: f64,
    msg: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    let bind_address = "127.0.0.1";
    info!("starting with bind address {}", bind_address);
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    info!("cors_layer: {:?}", cors_layer);
    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(cors_layer));
    let listener = tokio::net::TcpListener::bind(format!("{bind_address}:3000"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    let unix_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let response = JsonData {
        unix_ts,
        msg: "message from api".to_string(),
    };
    info!("response: {:?}", response);

    Json(response)
}
