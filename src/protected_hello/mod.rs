use axum::{
    extract::
    Extension,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use tracing::info;

use crate::models::User;

#[derive(Serialize)]
pub struct HelloResponse {
    message: String,
    status: String,
}

#[axum::debug_handler]
pub async fn hello(Extension(user): Extension<User>) -> impl IntoResponse {
    info!("Extra data from middleware - {:?}", user);
    let response = HelloResponse {
        message: format!("Hello, lets brainrot (This is protected - {:?})!", user),
        status: "success".to_string(),
    };

    Json(response)
}
