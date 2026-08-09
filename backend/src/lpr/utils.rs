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
    Dash,
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
        CharType::Dash => '-',
    }
}

/// Checks if a character matches the expected type.
fn matches_expected(c: char, expected: CharType) -> bool {
    match expected {
        CharType::Letter => c.is_ascii_alphabetic(),
        CharType::Number => c.is_ascii_digit(),
        CharType::Dash => c == '-',
    }
}

fn template_has_dash(template: &[CharType]) -> bool {
    template.contains(&CharType::Dash)
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

pub fn clean_numberplate(ocr_text: &str) -> String {
    let input_has_dash = ocr_text.contains('-') || ocr_text.contains('_');

    let sanitized_with_dash: String = ocr_text
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c.to_ascii_uppercase())
            } else if c == '-' || c == '_' || c == ' ' || c == '/' || c == '.' {
                Some('-')
            } else {
                None
            }
        })
        .collect();

    let sanitized_no_dash: String = sanitized_with_dash.chars().filter(|&c| c != '-').collect();

    use CharType::{Dash as D, Letter as L, Number as N};

    let templates: &[&[CharType]] = &[
        // --- UK Formats (Priority, No Hyphens) ---
        &[L, L, N, N, L, L, L], // Current (Since 2001) -> e.g., AB12 CDE
        &[L, N, N, N, L, L, L], // Prefix (1983-2001) -> e.g., A123 BCD
        &[L, L, L, N, N, N, L], // Suffix (1963-1983) -> e.g., ABC 123D
        &[L, L, L, N, N, N, N], // Northern Ireland -> e.g., ABC 1234
        &[L, L, L, N, N, N],    // Pre-1963 -> e.g., ABC 123
        &[N, N, N, L, L, L],    // Pre-1963 Reversed -> e.g., 123 ABC
        // --- European Formats (With Hyphens) ---
        &[L, L, D, N, N, N, D, L, L], // France / Italy (SIV) -> e.g., AA-123-AA
        &[N, N, D, L, L, D, N, N],    // Netherlands -> e.g., 12-AB-34
        &[L, L, D, N, N, D, L, L],    // Netherlands -> e.g., AB-12-CD
        &[N, N, D, L, L, L, D, N],    // Netherlands -> e.g., 12-ABC-3
        &[N, D, L, L, L, D, N, N],    // Netherlands -> e.g., 1-ABC-23
        &[L, L, D, N, N, N, D, L],    // Germany -> e.g., B-AB-123 / HH-AB-12
        &[L, D, L, L, D, N, N, N, N], // Germany -> e.g., B-AB-1234
        &[N, N, D, N, N, D, L, L],    // Portugal -> e.g., 12-34-AB
        &[L, L, L, D, N, N, N],       // Sweden / Finland -> e.g., ABC-123
        // --- European Formats (Without Hyphens) ---
        &[L, L, N, N, N, L, L], // France / Italy (without dashes) -> e.g., AA123AA
        &[N, N, N, N, L, L, L], // Spain -> e.g., 1234 ABC
        &[L, L, N, N, N, N],    // Generic EU -> e.g., AB1234
    ];

    // Evaluate template scores.
    // Tuple order for max_by_key:
    // 1. Percentage score (higher is better)
    // 2. Dash preference alignment (prefers dashed templates if input had hyphens)
    // 3. Lower index in `templates` array (UK templates win tie-breakers)
    let best = templates
        .iter()
        .enumerate()
        .filter_map(|(idx, &template)| {
            let has_dash = template_has_dash(template);
            let target_str = if has_dash {
                &sanitized_with_dash
            } else {
                &sanitized_no_dash
            };

            if target_str.len() == template.len() {
                let score = score_template(target_str, template);
                let percentage_score = (score * 1000) / template.len();
                let dash_preference_match = has_dash == input_has_dash;

                let key = (percentage_score, dash_preference_match, -(idx as isize));
                Some((key, target_str, template))
            } else {
                None
            }
        })
        .max_by_key(|&(key, _, _)| key);

    if let Some((_, target_str, template)) = best {
        return apply_template(target_str, template);
    }

    if input_has_dash {
        sanitized_with_dash
    } else {
        sanitized_no_dash
    }
}
