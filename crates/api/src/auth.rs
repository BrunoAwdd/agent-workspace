use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// JWT claims. `sub` is treated as the agent_id throughout the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — maps to agent_id.
    pub sub: String,
    /// Expiry (Unix timestamp).
    pub exp: usize,
    /// Optional display name used for auto-registration.
    pub name: Option<String>,
    /// Optional role used for auto-registration.
    pub role: Option<String>,
}

/// Axum middleware. Validates `Authorization: Bearer <jwt>` on every request.
///
/// If `JWT_SECRET` was not configured, auth is skipped entirely (dev mode).
/// On valid token, injects `Claims` into request extensions.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(ref secret) = state.jwt_secret else {
        return Ok(next.run(req).await);
    };

    let token = bearer_token(&req).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = validate_jwt(token, secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

fn validate_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<Claims>(token, &key, &validation).map(|d| d.claims)
}
