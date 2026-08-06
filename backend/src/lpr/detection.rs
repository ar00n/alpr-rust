use image::{imageops::FilterType, DynamicImage, GenericImageView, Rgb, RgbImage};
use ndarray::{Array, Axis, Ix3};
use ort::{
    inputs, session::{Session, builder::GraphOptimizationLevel}, sys::OrtLoggingLevel, value::Tensor,
};
use std::error::Error;

use crate::lpr::utils::get_onnx_providers;

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub score: f32,
    pub class_id: usize,
}

pub struct YoloDetector {
    session: Session,
    input_size: u32,
}

impl YoloDetector {
    pub fn new(model_path: &str, input_size: u32) -> Result<Self, Box<dyn Error>> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_execution_providers(get_onnx_providers())?
            .with_log_verbosity(OrtLoggingLevel::ORT_LOGGING_LEVEL_WARNING as i32)?
            .commit_from_file(model_path)?;
        Ok(Self {
            session,
            input_size,
        })
    }

    pub fn detect(&mut self, img: &DynamicImage) -> Result<Vec<BoundingBox>, Box<dyn Error>> {
        let (orig_width, orig_height) = img.dimensions();
        let output_img = img.to_rgb8();

        let scale = (self.input_size as f32 / orig_width as f32)
            .min(self.input_size as f32 / orig_height as f32);
        let new_w = (orig_width as f32 * scale).round() as u32;
        let new_h = (orig_height as f32 * scale).round() as u32;
        let pad_w = (self.input_size - new_w) / 2;
        let pad_h = (self.input_size - new_h) / 2;

        let mut padded_img =
            RgbImage::from_pixel(self.input_size, self.input_size, Rgb([114, 114, 114]));
        let resized_img = image::imageops::resize(&output_img, new_w, new_h, FilterType::Triangle);
        image::imageops::overlay(&mut padded_img, &resized_img, pad_w as i64, pad_h as i64);

        let img_data = padded_img.into_raw();
        let chw_array = Array::from_shape_vec(
            (self.input_size as usize, self.input_size as usize, 3),
            img_data,
        )?
        .permuted_axes([2, 0, 1]);

        let input_tensor = chw_array.mapv(|p| p as f32 / 255.0).insert_axis(Axis(0));

        let mut raw_boxes = Vec::new();
        let confidence_threshold = 0.8;
        let iou_threshold = 0.45;

        {
            let outputs = self
                .session
                .run(inputs!["images" => Tensor::from_array(input_tensor)?])?;
            let output_view = outputs["output0"]
                .try_extract_array::<f32>()?
                .into_dimensionality::<Ix3>()?;

            let max_det = output_view.shape()[1];
            let num_elements = output_view.shape()[2];

            for i in 0..max_det {
                if num_elements < 6 {
                    continue;
                }

                let score = output_view[[0, i, 4]];
                let class_id = output_view[[0, i, 5]] as usize;

                if score > confidence_threshold && class_id == 0 {
                    let x1 = output_view[[0, i, 0]];
                    let y1 = output_view[[0, i, 1]];
                    let x2 = output_view[[0, i, 2]];
                    let y2 = output_view[[0, i, 3]];

                    let orig_x1 = ((x1 - pad_w as f32) / scale)
                        .max(0.0)
                        .min(orig_width as f32);
                    let orig_y1 = ((y1 - pad_h as f32) / scale)
                        .max(0.0)
                        .min(orig_height as f32);
                    let orig_x2 = ((x2 - pad_w as f32) / scale)
                        .max(0.0)
                        .min(orig_width as f32);
                    let orig_y2 = ((y2 - pad_h as f32) / scale)
                        .max(0.0)
                        .min(orig_height as f32);

                    raw_boxes.push(BoundingBox {
                        x1: orig_x1,
                        y1: orig_y1,
                        x2: orig_x2,
                        y2: orig_y2,
                        score,
                        class_id,
                    });
                }
            }
        }

        Ok(self.apply_nms(raw_boxes, iou_threshold))
    }

    fn apply_nms(&self, mut boxes: Vec<BoundingBox>, iou_threshold: f32) -> Vec<BoundingBox> {
        boxes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        let mut keep: Vec<BoundingBox> = Vec::new();

        for current_box in boxes {
            let mut discard = false;
            for kept_box in &keep {
                if kept_box.class_id == current_box.class_id
                    && self.calculate_iou(&current_box, kept_box) > iou_threshold
                {
                    discard = true;
                    break;
                }
            }
            if !discard {
                keep.push(current_box);
            }
        }
        keep
    }

    fn calculate_iou(&self, box1: &BoundingBox, box2: &BoundingBox) -> f32 {
        let x1 = box1.x1.max(box2.x1);
        let y1 = box1.y1.max(box2.y1);
        let x2 = box1.x2.min(box2.x2);
        let y2 = box1.y2.min(box2.y2);

        let intersection = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
        let area1 = (box1.x2 - box1.x1) * (box1.y2 - box1.y1);
        let area2 = (box2.x2 - box2.x1) * (box2.y2 - box2.y1);

        if area1 + area2 - intersection == 0.0 {
            return 0.0;
        }
        intersection / (area1 + area2 - intersection)
    }
}
