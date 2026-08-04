use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};

// --- Imagen 3 API Request DTOs ---

#[derive(Serialize)]
struct ImagenInstance {
    prompt: String,
}

#[derive(Serialize)]
struct OutputOptions {
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Serialize)]
struct ImagenParameters {
    #[serde(rename = "sampleCount")]
    sample_count: u32,
    #[serde(rename = "aspectRatio")]
    aspect_ratio: String,
    #[serde(rename = "outputOptions")]
    output_options: OutputOptions,
}

#[derive(Serialize)]
struct ImagenRequest {
    instances: Vec<ImagenInstance>,
    parameters: ImagenParameters,
}

// --- Imagen 3 API Response DTOs ---

#[derive(Deserialize)]
struct ImagenPrediction {
    #[serde(rename = "bytesBase64Encoded")]
    bytes_base64_encoded: String,
}

#[derive(Deserialize)]
struct ImagenResponse {
    predictions: Option<Vec<ImagenPrediction>>,
}

// --- Main Automation Engine ---

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // 1. Get API Key from environment
    let api_key = env::var("GEMINI_API_KEY")
        .or_else(|_| env::var("GOOGLE_API_KEY"))
        .map_err(|_| anyhow!("GEMINI_API_KEY or GOOGLE_API_KEY must be set in environment or .env file"))?;

    // 2. Output folder setup
    let output_dir = Path::new("./output_assets");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // 3. Read input JSON file
    let json_path = "starter-pack.json";
    let json_content = fs::read_to_string(json_path)
        .map_err(|e| anyhow!("Failed to read {json_path}: {e}"))?;

    let root_json: Value = serde_json::from_str(&json_content)?;

    // Extract cosmetics array (supports wrapper object or raw array)
    let items = match root_json.get("cosmetics") {
        Some(Value::Array(arr)) => arr.clone(),
        _ if root_json.is_array() => root_json.as_array().unwrap().clone(),
        _ => return Err(anyhow!("Could not find 'cosmetics' array in JSON")),
    };

    println!("🚀 Starting image generation for {} items...", items.len());

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/imagen-3.0-generate-002:predict?key={}",
        api_key
    );

    for (index, item) in items.iter().enumerate() {
        // Parse basic product attributes safely
        let id = item.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("kc-item-{:03}", index + 1))
            .to_string();

        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Cosmetic Product");
        let brand = item.get("brand").and_then(|v| v.as_str()).unwrap_or("KoColor");
        let shade = item.get("shade_name").and_then(|v| v.as_str()).unwrap_or("");
        let formulation = item.get("formulation").and_then(|v| v.as_str()).unwrap_or("");
        let color_hex = item.get("color_hex").and_then(|v| v.as_str()).unwrap_or("");

        // 4. Safely extract filename using owned Strings (avoids borrow checker errors)
        let image_url = item.get("image_url").and_then(|v| v.as_str()).unwrap_or("");
        let expected_filename = match image_url.split('/').last() {
            Some(filename) if !filename.is_empty() => filename.to_string(),
            _ => format!("{}.webp", id),
        };

        // Swap .webp to .png for raw output
        let filename = expected_filename.replace(".webp", ".png");
        let file_path = output_dir.join(&filename);

        if file_path.exists() {
            println!("⏩ [{}/{}] Skipping existing image: {}", index + 1, items.len(), filename);
            continue;
        }

        // 5. Construct high-fidelity prompt
        let prompt = build_product_prompt(brand, name, shade, formulation, color_hex);
        println!("🎨 [{}/{}] Generating for '{}' ({})", index + 1, items.len(), name, brand);

        let payload = ImagenRequest {
            instances: vec![ImagenInstance { prompt }],
            parameters: ImagenParameters {
                sample_count: 1,
                aspect_ratio: "4:3".to_string(),
                output_options: OutputOptions {
                    mime_type: "image/png".to_string(),
                },
            },
        };

        // 6. Send POST request to Gemini Imagen endpoint
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ Failed API call for {id}: {error_text}");
            continue;
        }

        let api_res: ImagenResponse = response.json().await?;

        // 7. Decode Base64 and write image to disk
        if let Some(predictions) = api_res.predictions {
            if let Some(first_pred) = predictions.first() {
                let image_bytes = BASE64.decode(&first_pred.bytes_base64_encoded)?;
                fs::write(&file_path, image_bytes)?;
                println!("✅ Saved image: {}", file_path.display());
            }
        } else {
            eprintln!("⚠️ No image prediction returned for {id}");
        }

        // Rate limit safety delay (1.5 seconds)
        sleep(Duration::from_millis(1500)).await;
    }

    println!("🎉 All product images generated successfully!");
    Ok(())
}

/// Constructs a clean studio prompt for luxury cosmetic rendering
fn build_product_prompt(brand: &str, name: &str, shade: &str, formulation: &str, color_hex: &str) -> String {
    let mut details = Vec::new();
    if !shade.is_empty() { details.push(format!("Shade: {}", shade)); }
    if !formulation.is_empty() { details.push(format!("Texture/Formulation: {}", formulation)); }
    if !color_hex.is_empty() { details.push(format!("Dominant Color Tone: {}", color_hex)); }

    let extra_details = if details.is_empty() {
        String::new()
    } else {
        format!(" ({})", details.join(", "))
    };

    format!(
        "High-end minimalist commercial studio product photography of standard cosmetic package for {brand} {name}{extra_details}. \
        Displayed elegantly on a neutral travertine stone pedestal, soft realistic studio lighting, clean luxury beauty aesthetic, 8k resolution, sharp detail, neutral warm background.",
    )
}
