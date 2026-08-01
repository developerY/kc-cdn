use image::imageops::FilterType;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Instant;

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
