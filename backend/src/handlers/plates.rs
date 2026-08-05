use axum::{
    extract::{Query, State},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    models::{PlateRead, User},
    state::AppState,
};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub plate: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedHistoryResponse {
    pub items: Vec<PlateRead>,
    pub page: u64,
    pub per_page: u64,
    pub total: i64,
    pub total_pages: u64,
}

#[utoipa::path(
    get,
    path = "/api/history",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "Paginated list of plate reads", body = PaginatedHistoryResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "History"
)]
pub async fn get_history_handler(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Query(params): Query<PaginationParams>,
) -> Json<PaginatedHistoryResponse> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 100);

    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;

    let search_pattern = params.plate.as_ref().map(|p| format!("%{p}%"));

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM plate_reads 
        WHERE (?1 IS NULL OR plate LIKE ?1)
        "#,
        search_pattern
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let reads = sqlx::query_as!(
        PlateRead,
        r#"
        SELECT 
            id, 
            plate, 
            confidence, 
            snapshot_image, 
            timestamp as "timestamp: DateTime<Utc>", 
            was_allowed 
        FROM plate_reads 
        WHERE (?1 IS NULL OR plate LIKE ?1)
        ORDER BY id DESC 
        LIMIT ?2 OFFSET ?3
        "#,
        search_pattern,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let total_pages = (total as f64 / per_page as f64).ceil() as u64;

    Json(PaginatedHistoryResponse {
        items: reads,
        page,
        per_page,
        total,
        total_pages,
    })
}