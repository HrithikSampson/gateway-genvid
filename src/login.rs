use argon2::{PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use axum::{extract::{Json}, http::StatusCode};
use tracing::info;
use uuid::Uuid;
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::config::AppConfig;
use crate::schema::users::dsl::*;
use crate::connection::get_db_connection;
use crate::models::User;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub jwt_id: String,
}

fn is_valid_user(username_input: &str, password_input: &str) -> Result<Option<(i32, String, String)>, StatusCode> {
    let connection = &mut get_db_connection();
    let result = users
        .filter(name.eq(username_input))
        .select(User::as_select())
        .first(connection)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(user) = result {
        if let Some(ref db_password_hash) = user.password_hash {
            let parsed_hash = PasswordHash::new(db_password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if argon2::Argon2::default()
                .verify_password(password_input.as_bytes(), &parsed_hash).is_ok() {
                info!("User {} authenticated successfully", user.name);
                return Ok(Some((user.id, user.name.clone(), String::from(db_password_hash))));
            } else {
                eprintln!("Password mismatch for user: {}", user.name);
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

#[axum::debug_handler]
pub async fn loginhandler(
    jar: axum_extra::extract::CookieJar,
    Json(login_info): Json<LoginRequest>
) -> Result<(axum_extra::extract::CookieJar, Json<LoginResponse>), StatusCode> {
    info!("Login attempt for user: {}", login_info.username);

    match is_valid_user(&login_info.username, &login_info.password)? {
        Some((user_id, username, _)) => {
            info!("User attempt to login valid");
            let jwt_id = Uuid::new_v4().to_string();
            let claims = Claims {
                sub: user_id.to_string(),
                exp: (chrono::Utc::now() + chrono::Duration::minutes(AppConfig::instance().features.jwt_token_duration)).timestamp(),
                jwt_id: jwt_id.clone(),
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(AppConfig::instance().features.jwt_secret.as_bytes()),
            ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Set session cookie (JWT or session token)
            let jar = jar.add(axum_extra::extract::cookie::Cookie::new("refresh_token", jwt_id.clone()));
            let jar = jar.add(axum_extra::extract::cookie::Cookie::new("username", username));
            let jar = jar.add(axum_extra::extract::cookie::Cookie::new("user_id", user_id.to_string()));

            Ok((jar, Json(LoginResponse { token })))
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
