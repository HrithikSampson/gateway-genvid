use axum::{extract::{Request}, http::{StatusCode, header::AUTHORIZATION}, middleware::Next, response::{IntoResponse, Response}};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation, Header, errors::ErrorKind};
use tracing::info;
use uuid::Uuid;
use chrono::Utc;
use crate::{config::AppConfig, models::User};
use crate::schema::users::dsl::*;
use crate::login::Claims;
use crate::connection::get_db_connection;
use axum_extra::extract::CookieJar;

pub async fn auth_middleware_with_session(
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = req.headers().clone();
    let mut conn = get_db_connection();
    let access_token_opt = extract_bearer_token(&headers);
    let decoding_key = DecodingKey::from_secret(AppConfig::instance().features.jwt_secret.as_bytes());
    let validation = Validation::default();
    let session = jar.get("refresh_token").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(token) = access_token_opt {
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let id_of_user = token_data.claims.sub.clone();
                let user_id = id_of_user.parse::<i32>().map_err(|_| StatusCode::UNAUTHORIZED)?;
                let user_result = users
                    .find(user_id)
                    .select(User::as_select())
                    .first::<User>(&mut conn)
                    .optional();
                let result = user_result
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .ok_or_else(|| {
                        StatusCode::UNAUTHORIZED
                    })?;
                req.extensions_mut().insert(result);
                Ok(next.run(req).await)
            }
            Err(err) if matches!(err.kind(), ErrorKind::ExpiredSignature) => {
                info!("Access token expired. Attempting refresh using session...");
                let refresh_token_str = session.value();
                    match decode::<Claims>(&refresh_token_str, &decoding_key, &validation) {
                        Ok(refresh_data) => {
                            info!("Refresh token valid for user: {}", refresh_data.claims.sub);

                            let new_claims = Claims {
                                sub: refresh_data.claims.sub.clone(),
                                exp: (Utc::now() + chrono::Duration::minutes(AppConfig::instance().features.jwt_token_duration)).timestamp(),
                                jwt_id: Uuid::new_v4().to_string(),
                            };

                            match encode(
                                &Header::default(),
                                &new_claims,
                                &EncodingKey::from_secret(AppConfig::instance().features.jwt_secret.as_bytes()),
                            ) {
                                Ok(new_token) => {
                                    let mut response = next.run(req).await.into_response();
                                    use axum::http::header::HeaderValue;
                                    response.headers_mut().insert(
                                        AUTHORIZATION,
                                        HeaderValue::from_str(&format!("Bearer {}", new_token)).unwrap(),
                                    );
                                    Ok(response)
                                }
                                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                            }
                        }
                        Err(_) => Err(StatusCode::UNAUTHORIZED),
                    }
            }
            Err(err) => {
                info!("JWT decode error: {}", err);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers.get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}
