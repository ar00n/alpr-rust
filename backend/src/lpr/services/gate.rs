use futures_util::future::join_all;
use sqlx::{Pool, Sqlite};

use crate::{actions::execute_action, crypto::decrypt_data, error::AppError, models::CreateCustomAction};

pub async fn trigger_api(conn: &Pool<Sqlite>, encryption_key: &Vec<u8>, plate: &str) -> Result<(), AppError> {
    tracing::debug!("[GATE ACTUATION] Triggered open command for plate: {}", plate);

    let records = sqlx::query!(
        r#"
        SELECT name, method, url, auth_type, auth_data, headers, body_template, delay_seconds 
        FROM custom_actions
        "#
        )
        .fetch_all(conn)
        .await?;

    tracing::debug!("{:#?}", records);

    let mut actions = Vec::new();

    for rec in records {
        let headers_val = rec.headers.and_then(|h| serde_json::from_str(&h).ok());

        let decrypted_auth = match rec.auth_data.as_deref().filter(|s| !s.is_empty()) {
            Some(data) => match decrypt_data(data, encryption_key) {
                Ok(decrypted) => Some(decrypted),
                Err(err) => {
                    tracing::error!("Failed to decrypt auth_data for action '{}': {:?}", rec.name, err);
                    continue; 
                }
            },
            None => None,
        };

        let auth_data = match decrypted_auth {
            Some(data) => match serde_json::from_str(&data) {
                Ok(parsed) => Some(parsed),
                Err(err) => {
                    tracing::error!("Failed to parse decrypted auth_data JSON for action '{}': {:?}", rec.name, err);
                    continue; 
                }
            },
            None => None,
        };
        
        actions.push(CreateCustomAction {
            name: rec.name,
            method: rec.method,
            url: rec.url,
            auth_type: rec.auth_type,
            auth_data,
            headers: headers_val,
            body_template: rec.body_template,
            delay_seconds: rec.delay_seconds, // Map delay
        });
    }

    let futures = actions.iter().map(|action| async move {
        if let Some(delay) = action.delay_seconds {
            if delay > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(delay as u64)).await;
            }
        }

        execute_action(
            &action.url,
            &action.method,
            action.body_template.as_deref(),
            action.headers.as_ref(),
            &action.auth_type,
            action.auth_data.as_ref(),
            plate,
        ).await
    });

    let results = join_all(futures).await;

    for (action, result) in actions.iter().zip(results) {
        match result {
            Ok((status, response_body)) => {
                tracing::debug!(
                    "Action '{}' succeeded with status {}: {}",
                    action.name, status, response_body
                );
            }
            Err(err) => {
                tracing::error!("Action '{}' failed: {:?}", action.name, err);
            }
        }
    }

    Ok(())
}