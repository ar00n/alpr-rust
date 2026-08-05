use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, Algorithm, Validation};

use crate::{
    models::{Claims, User},
    state::AppState,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let (mut parts, body) = req.into_parts();

    let auth_header = TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Bearer token".into(),
            )
        })?;

    let token = auth_header.token();

    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::EdDSA];

    let token_data = decode::<Claims>(token, &state.jwt.decoding_key, &validation)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let claims = token_data.claims;

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", &claims.sub)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".into()))?;

    parts.extensions.insert(user);

    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}

pub async fn admin_middleware(
    Extension(user): Extension<User>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    if user.username != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin privileges required"));
    }

    Ok(next.run(req).await)
}
