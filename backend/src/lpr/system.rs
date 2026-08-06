use moka::sync::Cache;
use sqlx::{Pool, Sqlite};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinSet;

use crate::lpr::pipeline::LprPipeline;
use crate::lpr::services::{db, gate, snapshot};
use crate::models::{PlateEvent, PlateRead, VideoFrame};

pub fn start_lpr_system(
    mut frame_rx: watch::Receiver<Option<VideoFrame>>,
    plate_tx: broadcast::Sender<PlateRead>,
    db_pool: Pool<Sqlite>,
    encryption_key: Vec<u8>,
    snapshot_dir: String,
    config_rx: watch::Receiver<crate::models::PipelineConfig>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    let (event_tx, mut event_rx) = mpsc::channel::<PlateEvent>(100);

    // ---------------------------------------------------------
    // WORKER 1: Dedicated Inference Thread (Blocking)
    // ---------------------------------------------------------
    let mut shutdown_rx_w1 = shutdown_rx.resubscribe();

    tokio::task::spawn_blocking(move || {
        let mut pipeline = LprPipeline::new(
            "models/number-plate-yolo26n.onnx",
            "models/PP-OCRv6_small_rec_onnx.onnx",
            "models/ppocrv6_dict.txt",
        )
        .expect("Failed to initialize ONNX pipeline");

        let debounce_cache: Cache<String, ()> = Cache::builder()
            .time_to_idle(Duration::from_secs(30))
            .build();

        let rt = tokio::runtime::Handle::current();

        loop {
            // Unblock either when frame changes or shutdown signal is received
            let should_continue = rt.block_on(async {
                tokio::select! {
                    res = frame_rx.changed() => res.is_ok(),
                    _ = shutdown_rx_w1.recv() => false,
                }
            });

            if !should_continue {
                break;
            }

            let frame_opt = frame_rx.borrow_and_update().clone();
            let Some(frame) = frame_opt else { continue };

            match pipeline.recognize_plate_from_rgb(&frame.data, frame.width, frame.height) {
                Ok(Some((plate, confidence))) => {
                    if confidence < 0.5 {
                        tracing::debug!("⚠️ Low confidence plate read: {} ({:.2})", plate, confidence);
                        continue;
                    }

                    if debounce_cache.get(&plate).is_some() {
                        continue;
                    }

                    debounce_cache.insert(plate.clone(), ());

                    let event = PlateEvent {
                        plate,
                        confidence,
                        frame,
                    };

                    if event_tx.blocking_send(event).is_err() {
                        break;
                    }
                }
                Ok(None) => {}
                Err(e) => tracing::error!("⚠️ Inference Error: {}", e),
            }
        }
    });

    // ---------------------------------------------------------
    // WORKER 2: Business Logic Pipeline (Async)
    // ---------------------------------------------------------
    tokio::spawn(async move {
        // Track in-flight persistence and gate tasks so they aren't killed mid-flight
        let mut tasks = JoinSet::new();

        loop {
            tokio::select! {
                // Receive incoming plate events from Worker 1
                Some(event) = event_rx.recv() => {
                    let event_for_db = event.clone();
                    let db_for_db = db_pool.clone();
                    let plate_tx_clone = plate_tx.clone();
                    let snapshot_dir = snapshot_dir.clone();
                    
                    // Fetch latest trim config limit (handles if the config updates on the fly)
                    let trim_mb = config_rx.borrow().trim_snapshots_mb;
                    let trim_history_days = config_rx.borrow().trim_history_days;

                    // Branch A: Persistence Task
                    tasks.spawn(async move {
                        let snapshot_path = snapshot::save(&event_for_db.frame, &event_for_db.plate, &snapshot_dir).await;

                        if let Some(plate_read) = db::log_read(&event_for_db, snapshot_path, &db_for_db).await {
                            let _ = plate_tx_clone.send(plate_read);
                        }
                        
                        if let Some(trim_days) = trim_history_days {
                            db::trim_history(trim_days, &db_for_db, &snapshot_dir).await;
                        }

                        if let Some(trim_mb) = trim_mb {
                            snapshot::trim(&snapshot_dir, trim_mb).await;
                        }
                    });

                    // Branch B: Access Control Task
                    let event_for_gate = event.clone();
                    let db_for_gate = db_pool.clone();
                    let encryption_key_clone = encryption_key.clone();

                    tasks.spawn(async move {
                        let is_allowed = db::check_whitelist(&event_for_gate.plate, &db_for_gate).await;
                        if is_allowed {
                            let _ = gate::trigger_api(&db_for_gate, &encryption_key_clone, &event_for_gate.plate).await;
                        }
                    });
                }

                // Stop processing new events when shutdown is initiated
                _ = shutdown_rx.recv() => {
                    break;
                }

                // Worker 1 closed event_tx channel
                else => break,
            }
        }

        // Wait for all in-flight snapshot/DB/gate tasks to finish cleanly
        if !tasks.is_empty() {
            tracing::info!("Waiting for in-flight LPR tasks to complete...");
            while let Some(res) = tasks.join_next().await {
                if let Err(e) = res {
                    tracing::error!("Error in LPR background task during shutdown: {:?}", e);
                }
            }
        }
    })
}