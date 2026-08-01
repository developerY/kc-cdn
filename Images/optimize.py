import os
from PIL import Image

def optimize_images(source_dir, output_dir):
    # Ensure the output directory exists
    os.makedirs(output_dir, exist_ok=True)
    
    # Supported raw formats
    valid_extensions = ('.png', '.jpg', '.jpeg')
    
    for filename in os.listdir(source_dir):
        if filename.lower().endswith(valid_extensions):
            file_path = os.path.join(source_dir, filename)
            
            try:
                with Image.open(file_path) as img:
                    # Convert to RGB to ensure compatibility (strips alpha channel if present)
                    img = img.convert('RGB')
                    
                    # Resize to 512x512 using the high-quality Lanczos filter
                    resized_img = img.resize((512, 512), Image.Resampling.LANCZOS)
                    
                    # Construct the new WebP filename
                    base_name = os.path.splitext(filename)[0]
                    new_filename = f"{base_name}.webp"
                    output_path = os.path.join(output_dir, new_filename)
                    
                    # Save with WebP compression (Quality 80 is the sweet spot for ~30-40KB)
                    resized_img.save(output_path, 'WEBP', quality=80)
                    print(f"✅ Optimized: {filename} -> {new_filename}")
            
            except Exception as e:
                print(f"❌ Failed to process {filename}: {e}")

if __name__ == "__main__":
    # Define your local folders
    SOURCE_FOLDER = "./raw_assets"
    OUTPUT_FOLDER = "./assets"
    
    print("Starting optimization pipeline...")
    optimize_images(SOURCE_FOLDER, OUTPUT_FOLDER)
    print("Pipeline complete. Assets are ready to commit.")