use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// JWT claims. `sub` maps to agent_id; `scope` is space-separated OAuth2-style.
///
/// Example token payload:
/// ```json
/// { "sub": "agent-1", "exp": 9999999999, "scope": "tasks:read tasks:write messages:write" }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub name: Option<String>,
    pub role: Option<String>,
    /// Space-separated scopes, e.g. `"tasks:read tasks:write tasks:admin"`.
    /// `"*"` grants all scopes (used internally in dev mode).
    pub scope: Option<String>,
}

impl Claims {
    /// Wildcard claims injected in dev mode (no JWT_SECRET). Passes all scope checks.
    fn wildcard() -> Self {
        Claims {
            sub: "dev".to_string(),
            exp: usize::MAX,
            name: None,
            role: None,
            scope: Some("*".to_string()),
        }
    }
}

/// Validates `Authorization: Bearer <jwt>` on every protected request.
/// Dev mode (no JWT_SECRET): injects wildcard claims — all scopes pass.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = match state.jwt_secret {
        None => Claims::wildcard(),
        Some(ref secret) => {
            let token = bearer_token(&req).ok_or(StatusCode::UNAUTHORIZED)?;
            validate_jwt(token, secret).map_err(|_| StatusCode::UNAUTHORIZED)?
        }
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Per-route scope enforcement. Place after `require_auth` via `route_layer`.
/// Returns 403 if the token doesn't carry the required scope.
pub async fn check_scope(
    required: &'static str,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !has_scope(claims.scope.as_deref(), required) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// True if `scope` contains `required` or is the wildcard `"*"`.
fn has_scope(scope: Option<&str>, required: &str) -> bool {
    match scope {
        Some("*") => true,
        Some(s) => s.split_whitespace().any(|t| t == required),
        None => false,
    }
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
