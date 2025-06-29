use clap::Parser;
use image::{open, GrayImage, ImageBuffer};
use imageproc::contrast::equalize_histogram;
use imageproc::filter::gaussian_blur_f32;
use rayon::prelude::*;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image file paths
    #[arg(required = true)]
    input_paths: Vec<String>,
}

fn adaptive_threshold(image: &GrayImage, radius: f32, weight: f32) -> GrayImage {
    let blurred = gaussian_blur_f32(image, radius);
    let (width, height) = image.dimensions();
    let orig_buf = image.as_raw();
    let blur_buf = blurred.as_raw();
    let mut thresholded = ImageBuffer::new(width, height);
    let out_buf = thresholded.as_mut();

    out_buf
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, out_pixel)| {
            let orig = orig_buf[i] as f32;
            let blur = blur_buf[i] as f32;
            *out_pixel = if orig >= blur * (1.0 - weight) {
                255
            } else {
                0
            };
        });

    thresholded
}

fn main() -> Result<(), image::ImageError> {
    let args = Args::parse();

    args.input_paths
        .par_iter()
        .try_for_each(|input_path| -> Result<(), image::ImageError> {
            let mut img = open(input_path)?.to_luma8();

            // Apply CLAHE (Contrast Limited Adaptive Histogram Equalization)
            equalize_histogram(&mut img);

            // mean, stddevを1回のイテレーションで計算
            let (width, height) = img.dimensions();
            let min_side = width.min(height) as f32;
            let mut sum = 0.0f32;
            let mut sum_sq = 0.0f32;
            for &v in img.as_raw() {
                let fv = v as f32;
                sum += fv;
                sum_sq += fv * fv;
            }
            let npix = (width * height) as f32;
            let mean = sum / npix;
            let stddev = (sum_sq / npix - mean * mean).sqrt();
            let radius = (min_side / 30.0).max(5.0).min(30.0); // 5～30の範囲に制限
            let weight = if stddev < 30.0 {
                0.12
            } else if stddev < 60.0 {
                0.08
            } else {
                0.05
            };

            // Apply Sauvola's thresholding
            let thresholded = adaptive_threshold(&img, radius, weight);

            // Save the thresholded image
            let input_path_obj = Path::new(input_path);
            let parent_dir = input_path_obj.parent().unwrap();
            let file_stem = input_path_obj.file_stem().unwrap().to_str().unwrap();
            let output_path = parent_dir.join(format!("{}_thresholded.png", file_stem));
            thresholded.save(output_path)?;

            println!("Processed: {}", input_path);
            Ok::<(), image::ImageError>(())
        })?;

    Ok(())
}
