use axum::{
    extract::{FromRequestParts, Request, State},
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
    error::AppError, models::{Claims, User}, state::AppState,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();

    let auth_header = TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state)
        .await
        .map_err(|_| AppError::unauthorized("Missing or invalid Bearer token"))?;

    let token = auth_header.token();

    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::EdDSA];

    let token_data = decode::<Claims>(token, &state.jwt.decoding_key, &validation)
        .map_err(|e| AppError::unauthorized(e.to_string()))?;

    let claims = token_data.claims;

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", &claims.sub)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?
        .ok_or(AppError::unauthorized("User not found"))?;

    parts.extensions.insert(user);

    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}

pub async fn admin_middleware(
    Extension(user): Extension<User>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !user.is_admin {
        return Err(AppError::forbidden("Admin privileges required"));
    }

    Ok(next.run(req).await)
}
