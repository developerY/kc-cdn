use image::imageops::FilterType;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Instant;
// Ensure this is imported at the top for .dimensions() and .crop_imm()
use image::GenericImageView;

/*
 * Run:
 * cargo run --release
 * (Always run image processing tools in --release mode in Rust,
 * otherwise the lack of compiler optimizations will make the resizing
 * math run slowly).
 */

fn main() {
    let source_dir = "./raw_assets";
    let output_dir = "./assets";

    // Ensure output directory exists
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Read all files in the source directory
    let entries: Vec<_> = fs::read_dir(source_dir)
        .expect("Failed to read source directory")
        .filter_map(Result::ok)
        .collect();

    println!("🚀 Starting Rust parallel image optimization...");
    let start_time = Instant::now();

    // Process images concurrently using Rayon
    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "jpg" || ext_lower == "jpeg" || ext_lower == "png" {
                    optimize_image(&path, output_dir);
                }
            }
        }
    });

    println!("✅ Pipeline complete in {:.2?}. Assets are ready to commit.", start_time.elapsed());
}

fn optimize_image(input_path: &Path, output_dir: &str) {
    let file_name = input_path.file_stem().unwrap().to_str().unwrap();
    let output_path = Path::new(output_dir).join(format!("{}.webp", file_name));

    match image::open(input_path) {
        Ok(img) => {
            let (width, height) = img.dimensions();

            // 1. Calculate a 4:3 Landscape center crop
            let target_aspect = 4.0 / 3.0;
            let img_aspect = width as f32 / height as f32;

            let (crop_width, crop_height) = if img_aspect > target_aspect {
                // Original is wider than 4:3 (e.g., 16:9). Constrain by height.
                ((height as f32 * target_aspect).round() as u32, height)
            } else {
                // Original is taller than 4:3. Constrain by width.
                (width, (width as f32 / target_aspect).round() as u32)
            };

            let crop_x = (width - crop_width) / 2;
            let crop_y = (height - crop_height) / 2;

            let cropped_img = img.crop_imm(crop_x, crop_y, crop_width, crop_height);

            // 2. Resize to exact target dimensions (640 width x 480 height)
            println!("📏 Scaling {} to 640x480 landscape...", file_name);
            let resized = cropped_img.resize_exact(640, 480, FilterType::Lanczos3);

            // 3. Save as WebP
            match resized.save(&output_path) {
                Ok(_) => println!("   Converted: {} -> {}.webp", file_name, file_name),
                Err(e) => println!("❌ Failed to save {}: {}", file_name, e),
            }
        }
        Err(e) => println!("❌ Failed to open {}: {}", file_name, e),
    }
}

fn optimize_image_square_center(input_path: &Path, output_dir: &str) {
    let file_name = input_path.file_stem().unwrap().to_str().unwrap();
    let output_path = Path::new(output_dir).join(format!("{}.webp", file_name));

    match image::open(input_path) {
        Ok(img) => {
            let (width, height) = img.dimensions();

            // 1. Center crop to a perfect 1:1 square to prevent distortion
            let min_dim = width.min(height);
            let crop_x = (width - min_dim) / 2;
            let crop_y = (height - min_dim) / 2;
            let cropped_img = img.crop_imm(crop_x, crop_y, min_dim, min_dim);

            // 2. Resize to exact target dimensions
            let resized = cropped_img.resize_exact(512, 512, FilterType::Lanczos3);

            // 3. Save as WebP
            match resized.save(&output_path) {
                Ok(_) => println!("   Converted: {} -> {}.webp", file_name, file_name),
                Err(e) => println!("❌ Failed to save {}: {}", file_name, e),
            }
        }
        Err(e) => println!("❌ Failed to open {}: {}", file_name, e),
    }
}

fn optimize_image_square(input_path: &Path, output_dir: &str) {
    let file_name = input_path.file_stem().unwrap().to_str().unwrap();
    let output_path = Path::new(output_dir).join(format!("{}.webp", file_name));

    match image::open(input_path) {
        Ok(img) => {
            // Resize to 512x512
            let resized = img.resize_exact(512, 512, FilterType::Lanczos3);

            // Save as WebP
            match resized.save(&output_path) {
                Ok(_) => println!("   Converted: {} -> {}.webp", file_name, file_name),
                Err(e) => println!("❌ Failed to save {}: {}", file_name, e),
            }
        }
        Err(e) => println!("❌ Failed to open {}: {}", file_name, e),
    }
}
