use std::sync::Arc;
use std::time::Duration;

use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_pbutils::Discoverer;
use tokio::sync::{broadcast::{self, error::TryRecvError}, watch};

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

fn is_shutdown(rx: &mut broadcast::Receiver<()>) -> bool {
    match rx.try_recv() {
        Ok(()) | Err(TryRecvError::Closed) | Err(TryRecvError::Lagged(_)) => true,
        Err(TryRecvError::Empty) => false,
    }
}

fn interruptible_sleep(duration: Duration, rx: &mut broadcast::Receiver<()>) -> bool {
    let step = Duration::from_millis(50);
    let mut elapsed = Duration::from_millis(0);
    while elapsed < duration {
        if is_shutdown(rx) {
            return true;
        }
        std::thread::sleep(step);
        elapsed += step;
    }
    is_shutdown(rx)
}

pub fn start_rtsp_ingest(
    tx: broadcast::Sender<axum::body::Bytes>,
    pipeline_tx: watch::Sender<Option<VideoFrame>>,
    config_rx: watch::Receiver<PipelineConfig>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    gstreamer::init()?;

    loop {
        if is_shutdown(&mut shutdown_rx) {
            tracing::info!("RTSP ingest received shutdown signal. Exiting...");
            return Ok(());
        }

        let rtsp_url = match config_rx.borrow().rtsp_url.clone() {
            Some(url) => url,
            None => {
                if interruptible_sleep(Duration::from_secs(1), &mut shutdown_rx) {
                    tracing::info!("RTSP ingest shutting down while waiting for RTSP URL.");
                    return Ok(());
                }
                continue;
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
            Ok(p) => p
                .downcast::<gstreamer::Pipeline>()
                .expect("Expected a gst::Pipeline"),
            Err(e) => {
                tracing::error!("Failed to launch pipeline: {}. Retrying in 1s...", e);
                if interruptible_sleep(Duration::from_secs(1), &mut shutdown_rx) {
                    return Ok(());
                }
                continue;
            }
        };

        let rate_limiter = pipeline.by_name("rate_limiter").unwrap();

        let initial_fps = config_rx.borrow().fps;
        rate_limiter.set_property("max-rate", initial_fps as i32);

        let mut config_rx_bg = config_rx.clone();
        let mut shutdown_rx_bg = shutdown_rx.resubscribe();
        let rate_limiter_clone = rate_limiter.clone();
        let pipeline_clone = pipeline.clone();
        let current_rtsp_url = rtsp_url.clone();

        let bg_task = tokio::spawn(async move {
            let mut previous_fps = config_rx_bg.borrow().fps;

            loop {
                tokio::select! {
                    res = config_rx_bg.changed() => {
                        if res.is_err() {
                            break;
                        }
                        let config = config_rx_bg.borrow();

                        if config.rtsp_url != Some(current_rtsp_url.clone()) {
                            tracing::info!("RTSP URL changed. Sending EOS to cleanly restart pipeline...");
                            let _ = pipeline_clone.send_event(gstreamer::event::Eos::new());
                            break;
                        }

                        let new_fps = config.fps;
                        if new_fps != previous_fps {
                            tracing::info!("Updating GStreamer FPS limit to: {}", new_fps);
                            rate_limiter_clone.set_property("max-rate", new_fps as i32);
                            previous_fps = new_fps;
                        }
                    }

                    _ = shutdown_rx_bg.recv() => {
                        break;
                    }
                }
            }
        });

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
                    let _ = tx_clone.send(bytes);

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
                    let map = buffer
                        .map_readable()
                        .map_err(|_| gstreamer::FlowError::Error)?;

                    let s = caps.structure(0).ok_or(gstreamer::FlowError::Error)?;
                    let width = s
                        .get::<i32>("width")
                        .map_err(|_| gstreamer::FlowError::Error)? as i64;
                    let height = s
                        .get::<i32>("height")
                        .map_err(|_| gstreamer::FlowError::Error)? as i64;

                    let frame = VideoFrame {
                        data: Arc::from(map.as_slice()),
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
            let _ = pipeline.set_state(gstreamer::State::Null);
            bg_task.abort();
            if interruptible_sleep(Duration::from_secs(1), &mut shutdown_rx) {
                return Ok(());
            }
            continue;
        }

        tracing::debug!("GStreamer pipeline started for {}", rtsp_url);

        let bus = pipeline.bus().unwrap();

        loop {
            if is_shutdown(&mut shutdown_rx) {
                tracing::info!("🛑 RTSP ingest received shutdown signal. Stopping pipeline...");
                let _ = pipeline.send_event(gstreamer::event::Eos::new());
                let _ = pipeline.set_state(gstreamer::State::Null);
                bg_task.abort();
                return Ok(());
            }

            if let Some(msg) = bus.timed_pop(gstreamer::ClockTime::from_mseconds(250)) {
                use gstreamer::MessageView;
                match msg.view() {
                    MessageView::Error(err) => {
                        tracing::error!("GStreamer error: {} ({:?})", err.error(), err.debug());
                        break; // Break the bus loop to restart the pipeline
                    }
                    MessageView::Eos(..) => {
                        tracing::info!("GStreamer End of Stream");
                        break; // Break the bus loop to restart the pipeline
                    }
                    _ => (),
                }
            }

            if config_rx.borrow().rtsp_url != Some(rtsp_url.clone()) {
                tracing::info!("RTSP URL change detected in bus loop. Restarting pipeline...");
                break;
            }
        }

        tracing::warn!("Stream ended/failed. Cleaning up and restarting...");
        let _ = pipeline.set_state(gstreamer::State::Null);
        bg_task.abort();

        if interruptible_sleep(Duration::from_secs(1), &mut shutdown_rx) {
            tracing::info!("RTSP ingest received shutdown signal during retry sleep. Exiting...");
            return Ok(());
        }
    }
}