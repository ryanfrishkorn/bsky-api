use axum::{Json, Router, response::IntoResponse, routing::get};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(serde::Serialize)]
struct JsonData {
    unix_ts: f64,
    msg: String,
}

#[tokio::main]
async fn main() {
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(cors_layer));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    let unix_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let response = JsonData {
        unix_ts,
        msg: "adsf".to_string(),
    };
    Json(response)
}
