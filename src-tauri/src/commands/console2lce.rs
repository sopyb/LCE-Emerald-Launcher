use std::path::Path;
use std::fs;
use tauri::AppHandle;
use crate::console2lce;

#[tauri::command]
#[allow(non_snake_case)]
pub async fn import_world(
    _app: AppHandle,
    input_path: String,
    output_path: String,
    profile: Option<String>,
    preserve_entities: Option<bool>,
) -> Result<String, String> {
    eprintln!("[import_world] called");
    eprintln!("[import_world]   input_path: {:?}", input_path);
    eprintln!("[import_world]   output_path: {:?}", output_path);
    eprintln!("[import_world]   profile: {:?}", profile);

    let options = console2lce::ConversionOptions {
        profile: profile.unwrap_or_else(|| "large".to_string()),
        preserve_entities: preserve_entities.unwrap_or(false),
        ..Default::default()
    };

    let input = Path::new(&input_path);
    eprintln!("[import_world]   input exists: {:?}", input.exists());
    eprintln!("[import_world]   input is_dir: {:?}", input.is_dir());

    let output_parent = Path::new(&output_path).parent();
    if let Some(parent) = output_parent {
        eprintln!("[import_world]   output parent: {:?}", parent);
        if !parent.exists() {
            eprintln!("[import_world]   creating output parent dir...");
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory {:?}: {}", parent, e))?;
            eprintln!("[import_world]   output parent dir created");
        }
    }

    let input_lower = input_path.to_lowercase();
    let result = if input_lower.ends_with(".ms") {
        eprintln!("[import_world] detected .ms LCE save, copying directly");
        fs::copy(&input_path, &output_path)
            .map_err(|e| format!("Failed to copy .ms file: {}", e))?;
        console2lce::ConversionResult {
            success: true,
            message: "LCE save copied successfully".to_string(),
            chunk_count: 0,
            unknown_blocks: Vec::new(),
        }
    } else if input.is_dir() {
        eprintln!("[import_world] detected Java world directory");
        console2lce::convert_java_world_to_lce(&input_path, &output_path, &options)
            .map_err(|e| {
                eprintln!("[import_world] Java world conversion FAILED: {}", e);
                format!("Java world conversion failed: {}", e)
            })?
    } else {
        eprintln!("[import_world] detected Xbox 360 / savegame file");
        console2lce::convert_xbox360_save_to_lce(&input_path, &output_path, &options)
            .map_err(|e| {
                eprintln!("[import_world] conversion FAILED: {}", e);
                format!("Xbox/STFS conversion failed: {}", e)
            })?
    };

    eprintln!("[import_world] conversion succeeded, {} chunks", result.chunk_count);
    let mut msg = format!("World imported successfully!\nChunks converted: {}\nOutput: {}",
        result.chunk_count, output_path);

    if !result.unknown_blocks.is_empty() {
        msg.push_str(&format!("\n\nUnknown blocks (mapped to air):\n{}", result.unknown_blocks.join("\n")));
    }

    Ok(msg)
}
