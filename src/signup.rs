use axum::{extract::Json, http::StatusCode};
use axum_extra::extract::CookieJar;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::schema::users::dsl::*;
use crate::connection::get_db_connection;
use crate::models::{User, NewUser};
use crate::config::AppConfig;
use crate::helper::hash_password::hash_password;
use tracing::info;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub token: String,
}

#[axum::debug_handler]
pub async fn signup_handler(
    jar: CookieJar,
    Json(signup_info): Json<SignupRequest>
) -> Result<(CookieJar, Json<SignupResponse>), StatusCode> {
    info!("Signup attempt for user: {}", signup_info.username);

    let conn = &mut get_db_connection();

    // Check if user already exists
    let existing_user = users
        .filter(name.eq(&signup_info.username))
        .first::<User>(conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT); // 409
    }

    // Hash the password
    let hashed_password = hash_password(&signup_info.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_token_created = Uuid::new_v4().to_string();

    let new_user = NewUser {
        auth_type_or_provider: None,
        refresh_token: &refresh_token_created,
        credit: 0,
        name: &signup_info.username,
        stripe_customer_id: None,
        password_hash: Some(&hashed_password),
    };

    // Insert into DB
    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the newly created user to get the ID
    let created_user = users
        .filter(name.eq(&signup_info.username))
        .first::<User>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jwt_id = Uuid::new_v4().to_string();
    let claims = crate::login::Claims {
        sub: created_user.id.to_string(),
        exp: (chrono::Utc::now()
            + chrono::Duration::minutes(AppConfig::instance().features.jwt_token_duration))
        .timestamp(),
        jwt_id: jwt_id.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(AppConfig::instance().features.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = jar
        .add(axum_extra::extract::cookie::Cookie::new("refresh_token", jwt_id.clone()))
        .add(axum_extra::extract::cookie::Cookie::new("username", signup_info.username.clone()))
        .add(axum_extra::extract::cookie::Cookie::new("user_id", created_user.id.to_string()));

    Ok((jar, Json(SignupResponse { token })))
}
