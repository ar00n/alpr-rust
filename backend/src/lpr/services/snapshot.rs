use image::{ImageBuffer, Rgb, DynamicImage};
use crate::models::VideoFrame;
use webp::Encoder;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn save(frame: &VideoFrame, plate: &str, snapshot_dir: &str) -> Option<String> {
    let width = frame.width;
    let height = frame.height;
    let buffer = frame.buffer.clone();
    let plate_string = plate.to_string();
    let snapshot_dir = snapshot_dir.to_string();

    tokio::task::spawn_blocking(move || {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
            
        let filepath = format!("{snapshot_dir}/{}_{}.webp", plate_string, timestamp);

        let map = buffer.map_readable().ok()?;
        let img_buf = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, map.as_slice().to_vec())?;
        let dynamic_img = DynamicImage::ImageRgb8(img_buf);

        // Range: 0.0 (max compression / lowest quality) to 100.0 (best quality)
        let quality = 1.0; 

        let encoder = Encoder::from_image(&dynamic_img).ok()?;
        let encoded_webp = encoder.encode(quality);

        fs::write(&filepath, &*encoded_webp).ok()?;

        Some(format!("{}_{}.webp", plate_string, timestamp))
    })
    .await
    .ok()
    .flatten()
}

/// New function to enforce the directory size limit
pub async fn trim(snapshot_dir: &str, max_mb: u64) {
    if max_mb == 0 {
        return; // Guard to avoid wiping out the whole directory if set to 0.
    }

    let snapshot_dir = snapshot_dir.to_string();

    tokio::task::spawn_blocking(move || {
        let limit_bytes = max_mb * 1024 * 1024;
        let Ok(entries) = fs::read_dir(&snapshot_dir) else { return };

        let mut files = Vec::new();
        let mut total_size = 0;

        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let size = meta.len();
                    total_size += size;
                    
                    let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    files.push((entry.path(), size, modified));
                }
            }
        }

        if total_size <= limit_bytes {
            return;
        }

        files.sort_unstable_by_key(|&(_, _, modified)| modified);

        for (path, size, _) in files {
            if total_size <= limit_bytes {
                break;
            }
            if fs::remove_file(&path).is_ok() {
                total_size = total_size.saturating_sub(size);
            }
        }
    })
    .await
    .ok();
}