use image::{
    imageops::{unsharpen, FilterType},
    DynamicImage, GenericImageView,
};

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
