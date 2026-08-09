use std::error::Error;
use std::time::{Duration, Instant};

use image::{DynamicImage, ImageBuffer, Rgb};

use crate::lpr::detection::{BoundingBox, YoloDetector};
use crate::lpr::recognition::PaddleOcr;
use crate::lpr::utils::{self, clean_numberplate};

#[derive(Debug, Clone)]
struct CachedPlateLocation {
    bbox: BoundingBox,
    last_seen: Instant,
}

pub struct LprPipeline {
    detector: YoloDetector,
    recognizer: PaddleOcr,

    location_cache: Vec<CachedPlateLocation>,
    corner_tolerance_px: f32,
    idle_timeout: Duration,
}

impl LprPipeline {
    pub fn new(
        yolo_model_path: &str,
        ocr_model_path: &str,
        ocr_dict_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let detector = YoloDetector::new(yolo_model_path, 640)?;
        let recognizer = PaddleOcr::new(ocr_model_path, ocr_dict_path)?;
        Ok(Self {
            detector,
            recognizer,
            location_cache: Vec::new(),
            corner_tolerance_px: 30.0,
            idle_timeout: Duration::from_secs(30),
        })
    }

    pub fn recognize_plate_from_rgb(
        &mut self,
        frame: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        let expected_len = (width as usize) * (height as usize) * 3;
        if frame.len() != expected_len {
            return Err("buffer length mismatch".into());
        }

        let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, frame.to_vec())
                .ok_or("failed to construct image buffer from raw frame")?;

        let img = DynamicImage::ImageRgb8(img_buf);
        let bboxes = self.detector.detect(&img)?;

        let mut results = Vec::new();
        let now = Instant::now();

        self.location_cache
            .retain(|c| now.duration_since(c.last_seen) < self.idle_timeout);

        for bbox in &bboxes {
            let mut is_parked = false;
            for cached in &mut self.location_cache {
                if Self::is_same_location(bbox, &cached.bbox, self.corner_tolerance_px) {
                    cached.last_seen = now;
                    cached.bbox = bbox.clone();
                    is_parked = true;
                    break;
                }
            }

            if is_parked {
                continue;
            }

            if let Some(processed_crop) = utils::prepare_ocr_crop(&img, bbox) {
                if let Ok((text, confidence)) = self.recognizer.recognize(&processed_crop) {
                    let cleaned: String = clean_numberplate(&text);

                    if !cleaned.is_empty() {
                        results.push((cleaned, confidence));

                        self.location_cache.push(CachedPlateLocation {
                            bbox: bbox.clone(),
                            last_seen: now,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    fn is_same_location(b1: &BoundingBox, b2: &BoundingBox, tolerance: f32) -> bool {
        (b1.x1 - b2.x1).abs() <= tolerance
            && (b1.y1 - b2.y1).abs() <= tolerance
            && (b1.x2 - b2.x2).abs() <= tolerance
            && (b1.y2 - b2.y2).abs() <= tolerance
    }
}
