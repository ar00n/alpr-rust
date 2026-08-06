use axum::body::Bytes;
use jsonwebtoken::{DecodingKey, EncodingKey, Header};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, watch};

use crate::models::{PipelineConfig, PlateRead};

#[derive(Clone)]
pub struct JWT {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub header: Header,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub plate_tx: broadcast::Sender<PlateRead>,
    pub rtsp_tx: broadcast::Sender<Bytes>,
    pub pipeline_config_tx: watch::Sender<PipelineConfig>,
    pub jwt: JWT,
    pub encryption_key: Vec<u8>,
}
