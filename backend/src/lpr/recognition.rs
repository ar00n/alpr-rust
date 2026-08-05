use image::{imageops::FilterType, DynamicImage, GenericImageView};
use ndarray::{Array4, Ix3};
use ort::{
    ep, inputs, session::{Session, builder::GraphOptimizationLevel}, sys::OrtLoggingLevel, value::Tensor,
};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

pub struct PaddleOcr {
    session: Session,
    vocab: Vec<String>,
}

impl PaddleOcr {
    pub fn new(model_path: &str, vocab_path: &str) -> Result<Self, Box<dyn Error>> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_execution_providers([
                ep::CUDA::default().build(),
                ep::OpenVINO::default().build(),
            ])?
            .with_log_verbosity(OrtLoggingLevel::ORT_LOGGING_LEVEL_WARNING as i32)?
            .commit_from_file(model_path)?;

        let file = File::open(vocab_path)?;
        let reader = BufReader::new(file);
        let mut vocab = vec!["blank".to_string()]; // Index 0 is reserved for CTC Blank

        for line in reader.lines() {
            vocab.push(line?);
        }
        vocab.push(" ".to_string());

        Ok(Self { session, vocab })
    }

    pub fn recognize(&mut self, img: &DynamicImage) -> Result<(String, f32), Box<dyn Error>> {
        let target_height = 48;
        let max_width = 320;

        let (orig_width, orig_height) = img.dimensions();

        let ratio = orig_width as f32 / orig_height as f32;
        let mut new_width = (target_height as f32 * ratio).ceil() as u32;

        new_width = new_width.max(1).min(max_width);

        let resized = img.resize_exact(new_width, target_height, FilterType::Triangle);

        let mut input_array =
            Array4::<f32>::zeros((1, 3, target_height as usize, max_width as usize));

        for y in 0..target_height {
            for x in 0..new_width {
                // Only loop up to the actual scaled image width
                let pixel = resized.get_pixel(x, y);

                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;

                input_array[[0, 0, y as usize, x as usize]] = (b - 0.5) / 0.5; // Channel 0: Blue
                input_array[[0, 1, y as usize, x as usize]] = (g - 0.5) / 0.5; // Channel 1: Green
                input_array[[0, 2, y as usize, x as usize]] = (r - 0.5) / 0.5; // Channel 2: Red
            }
        }

        let outputs = self
            .session
            .run(inputs![Tensor::from_array(input_array)?])?;

        let output_view = outputs[0]
            .try_extract_array::<f32>()?
            .into_dimensionality::<Ix3>()?;
        let seq_len = output_view.shape()[1];
        let vocab_size = output_view.shape()[2];

        let mut raw_preds = Vec::with_capacity(seq_len);
        for t in 0..seq_len {
            let mut max_val = f32::MIN;
            let mut max_idx = 0;
            for c in 0..vocab_size {
                let val = output_view[[0, t, c]];
                if val > max_val {
                    max_val = val;
                    max_idx = c;
                }
            }
            raw_preds.push((max_idx, max_val));
        }

        let mut decoded_text = String::new();
        let mut confidences = Vec::new();
        let mut last_idx = usize::MAX;

        for (idx, score) in raw_preds {
            // Note: 0 is the CTC blank character.
            if idx != 0 && idx != last_idx {
                if let Some(char_str) = self.vocab.get(idx) {
                    decoded_text.push_str(char_str);
                    confidences.push(score);
                }
            }
            last_idx = idx; // Resets to 0 upon hitting a blank, enabling repeating chars
        }

        let confidence = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f32>() / confidences.len() as f32
        };

        Ok((decoded_text, confidence))
    }
}
