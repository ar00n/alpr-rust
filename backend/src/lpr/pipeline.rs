use std::error::Error;
use image::{DynamicImage, ImageBuffer, Rgb};

use crate::lpr::detection::YoloDetector;
use crate::lpr::recognition::PaddleOcr;
use crate::lpr::utils;

pub struct LprPipeline {
    detector: YoloDetector,
    recognizer: PaddleOcr,
}

impl LprPipeline {
    pub fn new(
        yolo_model_path: &str,
        ocr_model_path: &str,
        ocr_dict_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let detector = YoloDetector::new(yolo_model_path, 640)?;
        let recognizer = PaddleOcr::new(ocr_model_path, ocr_dict_path)?;
        Ok(Self { detector, recognizer })
    }

    pub fn recognize_plate_from_rgb(
        &mut self,
        frame: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Option<(String, f32)>, Box<dyn Error>> {
        let expected_len = (width as usize) * (height as usize) * 3;
        if frame.len() != expected_len {
            return Err("buffer length mismatch".into());
        }

        let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, frame.to_vec())
                .ok_or("failed to construct image buffer from raw frame")?;
        
        let img = DynamicImage::ImageRgb8(img_buf);
        let bboxes = self.detector.detect(&img)?;
        
        if bboxes.is_empty() {
            return Ok(None);
        }

        for bbox in &bboxes {
            if let Some(processed_crop) = utils::prepare_ocr_crop(&img, bbox) {
                if let Ok((text, confidence)) = self.recognizer.recognize(&processed_crop) {
                    
                    // 1. Remove non-alphanumeric chars (spaces, hyphens, linebreaks)
                    // 2. Capitalize everything
                    let cleaned: String = text
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                        .to_uppercase();

                    if !cleaned.is_empty() {
                        return Ok(Some((cleaned, confidence)));
                    }
                }
            }
        }
        Ok(None)
    }
}