mod auth;
mod db;
mod handlers;
mod lpr;
mod models;
mod openapi;
mod router;
mod rtsp;
mod shutdown;
mod state;
mod actions;
mod error;
mod crypto;

use axum::body::Bytes;
use tokio::sync::{broadcast, watch};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{
    auth::jwt::setup_jwt, crypto::get_or_create_key, lpr::system::start_lpr_system, models::{PipelineConfig, PlateRead, VideoFrame}, rtsp::start_rtsp_ingest, state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("alpr_backend=info,ort=warn"));
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let db = db::init_db().await?;
    let snapshot_dir = std::env::var("SNAPSHOT_DIR").unwrap_or_else(|_| "snapshots".to_string());
    std::fs::create_dir_all(&snapshot_dir).unwrap();

    let (plate_tx, _) = broadcast::channel::<PlateRead>(100);
    let (rtsp_tx, _) = broadcast::channel::<Option<Bytes>>(100);
    let (pipeline_frame_tx, pipeline_frame_rx) = watch::channel::<Option<VideoFrame>>(None);
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let fps: u32 = sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'processing_framerate'")
        .fetch_optional(&db)
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1);

    let rtsp_url = sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'rtsp_url'")
        .fetch_optional(&db)
        .await?;

    let trim_snapshots_mb: Option<u64> = sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'trim_snapshots_mb'")
        .fetch_optional(&db)
        .await?
        .map(|v| v.parse::<u64>().unwrap_or(0));

    let trim_history_days: Option<u64> = sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'trim_history_days'")
        .fetch_optional(&db)
        .await?
        .map(|v| v.parse::<u64>().unwrap_or(0));

    let min_confidence: f32 = sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'min_confidence'")
        .fetch_optional(&db)
        .await?
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.8);

    let (pipeline_config_tx, pipeline_config_rx) = watch::channel::<PipelineConfig>(PipelineConfig {
        fps,
        rtsp_url,
        trim_snapshots_mb,
        trim_history_days,
        min_confidence
    });

    let tx_clone = rtsp_tx.clone();
    let pipeline_tx_clone = pipeline_frame_tx.clone();
    let rtsp_shutdown_rx = shutdown_tx.subscribe();
    
    let pipeline_config_rx_rtsp = pipeline_config_rx.clone();
    let rtsp_handle = tokio::spawn(async move {
        if let Err(e) = start_rtsp_ingest(tx_clone, pipeline_tx_clone, pipeline_config_rx_rtsp, rtsp_shutdown_rx).await {
            tracing::error!("RTSP stream error: {:?}", e);
        }
    });

    let encryption_key = get_or_create_key().unwrap();

    let lpr_handle = start_lpr_system(
        pipeline_frame_rx,
        plate_tx.clone(),
        db.clone(),
        encryption_key.clone(),
        snapshot_dir.clone(),
        pipeline_config_rx,
        shutdown_tx.subscribe(),
    );

    let state = AppState {
        db: db.clone(),
        plate_tx,
        rtsp_tx,
        pipeline_config_tx,
        jwt: setup_jwt(),
        encryption_key,
    };

    let app = router::build_router(state);

    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    tracing::info!("🚀 Backend running on http://{}", server_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal(shutdown_tx.clone()))
        .await?;

    tracing::info!("Cleaning up active background channels and DB connections...");
    let _ = shutdown_tx.send(());
    let _ = lpr_handle.await;
    let _ = rtsp_handle.await;
    db.close().await;
    tracing::info!("👋 Shutdown complete. Goodbye!");
    
    Ok(())
}