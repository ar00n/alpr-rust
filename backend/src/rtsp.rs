use futures::stream::StreamExt;
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_pbutils::Discoverer;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::{broadcast, watch};

use crate::models::{PipelineConfig, VideoFrame};

pub async fn validate_rtsp_url(url: String, timeout_secs: u64) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        gstreamer::init().map_err(|e| e.to_string())?;

        let timeout = gstreamer::ClockTime::from_seconds(timeout_secs);
        let discoverer = Discoverer::new(timeout).map_err(|e| e.to_string())?;

        match discoverer.discover_uri(&url) {
            Ok(_info) => Ok(()),
            Err(e) => Err(format!("Failed to connect to RTSP stream: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Task joined failed: {}", e))?
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

        let current_fps = Arc::new(AtomicU32::new(config_rx.borrow().fps));

        let pipeline_str = format!(
            "rtspsrc location={} latency=100 ! decodebin ! videorate name=rate_limiter drop-only=true ! tee name=t \
             t. ! queue ! videoconvert ! video/x-raw,format=RGB ! appsink name=rust_sink drop=true max-buffers=1 \
             t. ! queue ! videoconvert ! jpegenc quality=20 ! appsink name=browser_sink drop=true max-buffers=1",
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

        // Clamp fps to a minimum of 1 to prevent GLib panics
        rate_limiter.set_property(
            "max-rate",
            current_fps.load(Ordering::Relaxed).max(1) as i32,
        );

        let browser_sink = pipeline.by_name("browser_sink").unwrap();
        let browser_appsink = browser_sink.downcast::<AppSink>().unwrap();

        let tx_clone = tx.clone();
        let fps_browser = current_fps.clone(); // Lock-free clone

        browser_appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;

                    if fps_browser.load(Ordering::Relaxed) == 0 {
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

        let fps_rust = current_fps.clone();
        let pipeline_tx_clone = pipeline_tx.clone();

        let mut cached_caps: Option<(gstreamer::Caps, u32, u32)> = None;

        rust_appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;

                    if fps_rust.load(Ordering::Relaxed) == 0 {
                        return Ok(gstreamer::FlowSuccess::Ok);
                    }

                    let caps = sample.caps().ok_or(gstreamer::FlowError::Error)?;

                    let (width, height) = if let Some((ref c, w, h)) = cached_caps {
                        if c.as_ref() == caps {
                            (w, h)
                        } else {
                            let s = caps.structure(0).ok_or(gstreamer::FlowError::Error)?;
                            let w = s
                                .get::<i32>("width")
                                .map_err(|_| gstreamer::FlowError::Error)?
                                as u32;
                            let h = s
                                .get::<i32>("height")
                                .map_err(|_| gstreamer::FlowError::Error)?
                                as u32;
                            cached_caps = Some((caps.to_owned(), w, h));
                            (w, h)
                        }
                    } else {
                        let s = caps.structure(0).ok_or(gstreamer::FlowError::Error)?;
                        let w = s
                            .get::<i32>("width")
                            .map_err(|_| gstreamer::FlowError::Error)?
                            as u32;
                        let h = s
                            .get::<i32>("height")
                            .map_err(|_| gstreamer::FlowError::Error)?
                            as u32;
                        cached_caps = Some((caps.to_owned(), w, h));
                        (w, h)
                    };

                    let buffer = sample.buffer().ok_or(gstreamer::FlowError::Error)?;

                    let frame = VideoFrame {
                        buffer: buffer.to_owned(),
                        width,
                        height,
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
        let mut bus_stream = bus.stream();

        loop {
            tokio::select! {
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

                res = config_rx.changed() => {
                    if res.is_err() { break; }
                    let config = config_rx.borrow();

                    if config.rtsp_url != Some(rtsp_url.clone()) {
                        tracing::info!("RTSP URL changed. Signalling pipeline restart...");
                        break;
                    }

                    let previous_fps = current_fps.load(Ordering::Relaxed);
                    if config.fps != previous_fps {
                        tracing::info!("Updating GStreamer FPS limit to: {}", config.fps);
                        rate_limiter.set_property("max-rate", config.fps.max(1) as i32);
                        current_fps.store(config.fps, Ordering::Relaxed);
                    }
                }

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

fn stop_pipeline(pipeline: &gstreamer::Pipeline, pipeline_tx: &watch::Sender<Option<VideoFrame>>) {
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
