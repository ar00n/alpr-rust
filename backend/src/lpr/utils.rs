use image::{
    imageops::{unsharpen, FilterType},
    DynamicImage, GenericImageView,
};
use ort::ep::{self, ExecutionProviderDispatch};

use crate::lpr::detection::BoundingBox;

pub fn prepare_ocr_crop(img: &DynamicImage, bbox: &BoundingBox) -> Option<DynamicImage> {
    let (orig_width, orig_height) = img.dimensions();
    let rx1 = (bbox.x1.round() as u32).clamp(0, orig_width.saturating_sub(1));
    let ry1 = (bbox.y1.round() as u32).clamp(0, orig_height.saturating_sub(1));
    let rx2 = (bbox.x2.round() as u32).clamp(0, orig_width.saturating_sub(1));
    let ry2 = (bbox.y2.round() as u32).clamp(0, orig_height.saturating_sub(1));

    let padding = 10;
    let pad_x1 = rx1
        .saturating_sub(padding)
        .clamp(0, orig_width.saturating_sub(1));
    let pad_y1 = ry1
        .saturating_sub(padding)
        .clamp(0, orig_height.saturating_sub(1));
    let pad_x2 = (rx2 + padding).clamp(0, orig_width.saturating_sub(1));
    let pad_y2 = (ry2 + padding).clamp(0, orig_height.saturating_sub(1));

    let crop_width = pad_x2.saturating_sub(pad_x1);
    let crop_height = pad_y2.saturating_sub(pad_y1);

    if crop_width == 0 || crop_height == 0 {
        return None;
    }

    let cropped_subimg = img.crop_imm(pad_x1, pad_y1, crop_width, crop_height);
    let cropped_dynamic = DynamicImage::ImageRgba8(cropped_subimg.to_rgba8());

    let upscaled = cropped_dynamic.resize(crop_width * 3, crop_height * 3, FilterType::Lanczos3);

    let grayscale = upscaled.grayscale();
    let sharpened = unsharpen(&grayscale.to_luma8(), 2.0, 20);

    Some(DynamicImage::ImageRgb8(
        image::DynamicImage::ImageLuma8(sharpened).to_rgb8(),
    ))
}

pub fn get_onnx_providers() -> Vec<ExecutionProviderDispatch> {
    let mut providers = Vec::new();

    let enable_openvino = std::env::var("ENABLE_OPENVINO")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if enable_openvino {
        providers.push(ep::OpenVINO::default().build());
    }

    let enable_cuda = std::env::var("ENABLE_CUDA")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if enable_cuda {
        providers.push(ep::CUDA::default().build());
    }

    return providers;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CharType {
    Letter,
    Number,
}

fn fix_char(c: char, expected: CharType) -> char {
    match expected {
        CharType::Letter => match c {
            '0' => 'O',
            '1' => 'I',
            '2' => 'Z',
            '3' => 'J',
            '4' => 'A',
            '5' => 'S',
            '6' => 'G',
            '7' => 'T',
            '8' => 'B',
            _ => c,
        },
        CharType::Number => match c {
            'O' | 'Q' | 'D' => '0',
            'I' | 'L' => '1',
            'Z' => '2',
            'J' => '3',
            'A' => '4',
            'S' => '5',
            'G' => '6',
            'T' => '7',
            'B' => '8',
            _ => c,
        },
    }
}

/// Checks if a character matches the expected type.
fn matches_expected(c: char, expected: CharType) -> bool {
    match expected {
        CharType::Letter => c.is_ascii_alphabetic(),
        CharType::Number => c.is_ascii_digit(),
    }
}

fn score_template(plate: &str, template: &[CharType]) -> usize {
    plate
        .chars()
        .zip(template.iter())
        .filter(|&(c, &expected)| matches_expected(c, expected))
        .count()
}

fn apply_template(plate: &str, template: &[CharType]) -> String {
    plate
        .chars()
        .zip(template.iter())
        .map(|(c, &expected)| fix_char(c, expected))
        .collect()
}

pub fn clean_uk_numberplate(ocr_text: &str) -> String {
    let sanitized: String = ocr_text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if sanitized.len() == 7 {
        use CharType::{Letter as L, Number as N};

        // Format 1: Current (Since 2001) -> e.g., AB12 CDE
        let current_format = [L, L, N, N, L, L, L];
        // Format 2: Prefix (1983-2001) -> e.g., A123 BCD
        let prefix_format = [L, N, N, N, L, L, L];
        // Format 3: Suffix (1963-1983) -> e.g., ABC 123D
        let suffix_format = [L, L, L, N, N, N, L];

        let templates = [
            &current_format[..], // Checked first, wins tie-breakers
            &prefix_format[..],
            &suffix_format[..],
        ];

        let best_template = templates
            .iter()
            .max_by_key(|&&t| score_template(&sanitized, t))
            .unwrap();

        return apply_template(&sanitized, best_template);
    }

    sanitized
}