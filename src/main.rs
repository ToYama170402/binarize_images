use clap::Parser;
use image::{open, GrayImage, ImageBuffer, Luma};
use imageproc::contrast::equalize_histogram;
use imageproc::filter::gaussian_blur_f32;
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
    let mut thresholded = ImageBuffer::new(image.width(), image.height());
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y)[0] as f32;
            let blurred_pixel = blurred.get_pixel(x, y)[0] as f32;
            if pixel >= blurred_pixel * (1.0 - weight) {
                thresholded.put_pixel(x, y, Luma([255]));
            } else {
                thresholded.put_pixel(x, y, Luma([0]));
            }
        }
    }
    thresholded
}

fn main() -> Result<(), image::ImageError> {
    let args = Args::parse();

    for input_path in args.input_paths {
        let mut img = open(&input_path)?.to_luma8();

        // Apply CLAHE (Contrast Limited Adaptive Histogram Equalization)
        equalize_histogram(&mut img);

        // Apply Sauvola's thresholding
        let thresholded = adaptive_threshold(&img, 11.0, 0.05);

        // Save the thresholded image
        let input_path_obj = Path::new(&input_path);
        let parent_dir = input_path_obj.parent().unwrap();
        let file_stem = input_path_obj.file_stem().unwrap().to_str().unwrap();
        let output_path = parent_dir.join(format!("{}_thresholded.png", file_stem));
        thresholded.save(output_path)?;

        println!("Processed: {}", input_path);
    }

    Ok(())
}
