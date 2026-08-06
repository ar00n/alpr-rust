use futures::stream::StreamExt; // Required for bus_stream.next()
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_pbutils::Discoverer;
use tokio::sync::{broadcast, watch};
use std::time::Duration;

use crate::models::{PipelineConfig, VideoFrame};

pub fn validate_rtsp_url(url: &str, timeout_secs: u64) -> Result<(), String> {
    gstreamer::init().map_err(|e| e.to_string())?;

    let timeout = gstreamer::ClockTime::from_seconds(timeout_secs);
    let discoverer = Discoverer::new(timeout).map_err(|e| e.to_string())?;

    match discoverer.discover_uri(url) {
        Ok(_info) => Ok(()),
        Err(e) => Err(format!("Failed to connect to RTSP stream: {}", e)),
    }
}

pub async fn start_rtsp_ingest(
    tx: broadcast::Sender<Option<axum::body::Bytes>>,
    pipeline_tx: watch::Sender<Option<VideoFrame>>,
    mut config_rx: watch::Receiver<PipelineConfig>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    gstreamer::init()?;

    loop {
        let rtsp_url = loop {
            if let Some(url) = config_rx.borrow().rtsp_url.clone() {
                break url;
            }
            
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("RTSP ingest shutting down...");
                    let _ = pipeline_tx.send(None);
                    return Ok(());
                }
                res = config_rx.changed() => {
                    if res.is_err() { return Ok(()); }
                }
            }
        };

        tracing::info!("Starting/Restarting GStreamer pipeline for {}", rtsp_url);

        let pipeline_str = format!(
            "rtspsrc location={} latency=100 ! decodebin ! videorate name=rate_limiter drop-only=true ! tee name=t \
             t. ! queue ! videoconvert ! video/x-raw,format=RGB ! appsink name=rust_sink drop=true max-buffers=1 \
             t. ! queue ! videoconvert ! jpegenc ! appsink name=browser_sink drop=true max-buffers=1",
            rtsp_url
        );

        let pipeline = match gstreamer::parse::launch(&pipeline_str) {
            Ok(p) => p.downcast::<gstreamer::Pipeline>().unwrap(),
            Err(e) => {
                tracing::error!("Failed to launch pipeline: {}. Retrying in 1s...", e);
                let _ = pipeline_tx.send(None);
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(1)) => continue,
                    _ = shutdown_rx.recv() => return Ok(()),
                }
            }
        };

        let rate_limiter = pipeline.by_name("rate_limiter").unwrap();
        let mut previous_fps = config_rx.borrow().fps;
        
        // Clamp fps to a minimum of 1 to prevent GLib panics
        rate_limiter.set_property("max-rate", previous_fps.max(1) as i32);

        let browser_sink = pipeline.by_name("browser_sink").unwrap();
        let browser_appsink = browser_sink.downcast::<AppSink>().unwrap();

        let tx_clone = tx.clone();
        let config_rx_browser = config_rx.clone();
        browser_appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;

                    if config_rx_browser.borrow().fps == 0 {
                        return Ok(gstreamer::FlowSuccess::Ok);
                    }

                    let buffer = sample.buffer().ok_or(gstreamer::FlowError::Error)?;
                    let map = buffer
                        .map_readable()
                        .map_err(|_| gstreamer::FlowError::Error)?;

                    let bytes = axum::body::Bytes::copy_from_slice(map.as_slice());
                    let _ = tx_clone.send(Some(bytes));

                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        let rust_sink = pipeline.by_name("rust_sink").unwrap();
        let rust_appsink = rust_sink.downcast::<AppSink>().unwrap();

        let config_rx_rust = config_rx.clone();
        let pipeline_tx_clone = pipeline_tx.clone();
        rust_appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;

                    if config_rx_rust.borrow().fps == 0 {
                        return Ok(gstreamer::FlowSuccess::Ok);
                    }

                    let caps = sample.caps().ok_or(gstreamer::FlowError::Error)?;
                    let buffer = sample.buffer().ok_or(gstreamer::FlowError::Error)?;

                    let s = caps.structure(0).ok_or(gstreamer::FlowError::Error)?;
                    let width = s
                        .get::<i32>("width")
                        .map_err(|_| gstreamer::FlowError::Error)? as i64;
                    let height = s
                        .get::<i32>("height")
                        .map_err(|_| gstreamer::FlowError::Error)? as i64;

                    let frame = VideoFrame {
                        buffer: buffer.to_owned(),
                        width: width as u32,
                        height: height as u32,
                    };

                    let _ = pipeline_tx_clone.send(Some(frame));

                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        if let Err(err) = pipeline.set_state(gstreamer::State::Playing) {
            tracing::error!("Failed to set pipeline to Playing: {}. Retrying...", err);
            stop_pipeline(&pipeline, &pipeline_tx);
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(1)) => continue,
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }

        let bus = pipeline.bus().unwrap();
        let mut bus_stream = bus.stream(); // Create an async Stream of Gstreamer messages

        // 2. The unified main async event loop
        loop {
            tokio::select! {
                // Event A: Gstreamer Bus Messages
                msg = bus_stream.next() => {
                    let Some(msg) = msg else { break };
                    match msg.view() {
                        gstreamer::MessageView::Error(err) => {
                            tracing::error!("GStreamer error: {} ({:?})", err.error(), err.debug());
                            break;
                        }
                        gstreamer::MessageView::Eos(..) => {
                            tracing::info!("GStreamer End of Stream");
                            break;
                        }
                        _ => (),
                    }
                }

                // Event B: Config Changes
                res = config_rx.changed() => {
                    if res.is_err() { break; } // Channel closed
                    let config = config_rx.borrow();

                    if config.rtsp_url != Some(rtsp_url.clone()) {
                        tracing::info!("RTSP URL changed. Signalling pipeline restart...");
                        break;
                    }

                    if config.fps != previous_fps {
                        tracing::info!("Updating GStreamer FPS limit to: {}", config.fps);
                        // Prevent panic by ensuring max-rate >= 1
                        rate_limiter.set_property("max-rate", config.fps.max(1) as i32);
                        previous_fps = config.fps;
                    }
                }

                // Event C: App Shutdown
                _ = shutdown_rx.recv() => {
                    tracing::info!("🛑 RTSP ingest received shutdown signal. Stopping pipeline...");
                    let _ = tx.send(None); 
                    stop_pipeline(&pipeline, &pipeline_tx);
                    return Ok(());
                }
            }
        }

        tracing::warn!("Stream ended/failed or config changed. Cleaning up...");
        stop_pipeline(&pipeline, &pipeline_tx);

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => continue,
            _ = shutdown_rx.recv() => return Ok(()),
        }
    }
}

// Updated Stop Helper
fn stop_pipeline(
    pipeline: &gstreamer::Pipeline,
    pipeline_tx: &watch::Sender<Option<VideoFrame>>,
) {
    if let Some(browser_sink) = pipeline.by_name("browser_sink") {
        if let Ok(browser_appsink) = browser_sink.downcast::<AppSink>() {
            browser_appsink.set_callbacks(gstreamer_app::AppSinkCallbacks::builder().build());
        }
    }
    if let Some(rust_sink) = pipeline.by_name("rust_sink") {
        if let Ok(rust_appsink) = rust_sink.downcast::<AppSink>() {
            rust_appsink.set_callbacks(gstreamer_app::AppSinkCallbacks::builder().build());
        }
    }

    let _ = pipeline.set_state(gstreamer::State::Null);
    let _ = pipeline_tx.send(None);
}