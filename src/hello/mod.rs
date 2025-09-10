use axum::response::{IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HelloResponse {
    message: String,
    status: String,
}

#[axum::debug_handler]
pub async fn hello() -> impl IntoResponse {
    let response = HelloResponse {
        message: "Hello, lets brainrot!".to_string(),
        status: "success".to_string(),
    };

    Json(response)
}
