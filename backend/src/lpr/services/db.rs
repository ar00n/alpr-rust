use crate::models::{PlateEvent, PlateRead};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};

pub async fn log_read(
    event: &PlateEvent,
    snapshot_path: Option<String>,
    db: &Pool<Sqlite>,
) -> Option<PlateRead> {
    let result = sqlx::query_as!(
        PlateRead,
        r#"
        INSERT INTO plate_reads (plate, confidence, snapshot_image, was_allowed)
        WITH input(plate, confidence, snapshot_image) AS (
            VALUES (?, ?, ?)
        )
        SELECT 
            plate, 
            confidence, 
            snapshot_image,
            EXISTS (
                SELECT 1 
                FROM allow_list 
                WHERE allow_list.plate = input.plate 
                AND (expiry_date IS NULL OR expiry_date > CURRENT_TIMESTAMP)
            ) AS was_allowed
        FROM input
        RETURNING id, plate, confidence, snapshot_image, timestamp AS "timestamp: DateTime<Utc>", was_allowed;
        "#,
        &event.plate,
        event.confidence,
        snapshot_path
    )
    .fetch_one(db)
    .await;

    match result {
        Ok(read) => {
            tracing::debug!(
                "[DB] Logged plate to history: {} (ID: {:?})",
                read.plate,
                read.id
            );
            Some(read)
        }
        Err(e) => {
            tracing::error!("[DB] Failed to insert plate read: {:?}", e);
            None
        }
    }
}

pub async fn check_whitelist(plate: &str, db: &Pool<Sqlite>) -> bool {
    let result = sqlx::query_scalar!(
        r#"
        SELECT 1 FROM allow_list 
        WHERE plate = ? AND (expiry_date IS NULL OR expiry_date > CURRENT_TIMESTAMP)
        "#,
        plate
    )
    .fetch_optional(db)
    .await;

    let is_allowed = result.unwrap_or(None).is_some();
    tracing::debug!("[ACCESS] Plate {} whitelist status: {}", plate, is_allowed);
    is_allowed
}

pub async fn trim_history(days: u64, db: &Pool<Sqlite>, snapshot_dir: &str) {
    if days == 0 {
        return;
    }

    let modifier = format!("-{} days", days);

    let result = sqlx::query!(
        r#"
        DELETE FROM plate_reads 
        WHERE timestamp < datetime('now', ?)
        RETURNING snapshot_image
        "#,
        modifier
    )
    .fetch_all(db)
    .await;

    match result {
        Ok(deleted_rows) => {
            if !deleted_rows.is_empty() {
                tracing::debug!(
                    "[DB] Trimmed {} old plate read(s) from history.",
                    deleted_rows.len()
                );
                for row in deleted_rows {
                    if let Some(path) = row.snapshot_image {
                        let filepath = format!("{snapshot_dir}/{}", path);
                        if let Err(e) = tokio::fs::remove_file(&filepath).await {
                            tracing::warn!(
                                "[DB] Failed to remove orphaned snapshot file {}: {}",
                                filepath,
                                e
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("[DB] Failed to trim history: {:?}", e);
        }
    }
}
