use crate::kernel::auth::Token;
use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub device: String,
    pub sub: String,
    // TODO
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthError {
    Missing,
    Invalid,
    Expired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::Missing => (axum::http::StatusCode::UNAUTHORIZED, "Missing token"),
            AuthError::Invalid => (axum::http::StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::Expired => (axum::http::StatusCode::UNAUTHORIZED, "Expired token"),
        };
        let body = axum::Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl<S> FromRequestParts<S> for UserData
where
    S: Send + Sync + 'static,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::Missing)?;
        let token_str = token.0.token();
        let token_data = Token::from_str(token_str).map_err(|_| AuthError::Invalid)?;
        Ok(UserData {
            device: token_data.device,
            sub: token_data.sub,
            // TODO
            permissions: String::new(),
        })
    }
}
