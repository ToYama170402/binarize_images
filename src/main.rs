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

    args.input_paths.par_iter().try_for_each(|input_path| -> Result<(), image::ImageError> {
        let mut img = open(input_path)?.to_luma8();

        // Apply CLAHE (Contrast Limited Adaptive Histogram Equalization)
        equalize_histogram(&mut img);

        // Apply Sauvola's thresholding
        let thresholded = adaptive_threshold(&img, 11.0, 0.05);

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
