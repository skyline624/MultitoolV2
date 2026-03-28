use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;
use tauri::path::PathResolver;
use tauri::Manager;
use tauri::Runtime;

/// Structure pour stocker les chemins personnalisés de Star Citizen
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomGamePaths {
    pub paths: Vec<String>,
}

fn get_game_paths_config_file_path(path: &PathResolver<impl Runtime>) -> Result<PathBuf, String> {
    let config_dir = path.app_config_dir().map_err(|e| {
        format!("Impossible d'obtenir le répertoire de configuration de l'application: {}", e)
    })?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    }
    Ok(config_dir.join("game_paths.json"))
}

/// Sauvegarde les chemins personnalisés de Star Citizen
#[command]
pub fn save_custom_game_paths(app: tauri::AppHandle, paths: Vec<String>) -> Result<(), String> {
    let config_path = get_game_paths_config_file_path(app.path()).map_err(|e| e.to_string())?;
    let data = CustomGamePaths { paths };
    let json_data = serde_json::to_string(&data).map_err(|e| e.to_string())?;
    fs::write(config_path, json_data).map_err(|e| e.to_string())
}

/// Charge les chemins personnalisés de Star Citizen
#[command]
pub fn load_custom_game_paths(app: tauri::AppHandle) -> Result<CustomGamePaths, String> {
    let config_path = get_game_paths_config_file_path(app.path()).map_err(|e| e.to_string())?;

    if !config_path.exists() {
        return Ok(CustomGamePaths { paths: Vec::new() });
    }

    let json_data = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let data: CustomGamePaths = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;

    Ok(data)
}

/// Efface les chemins personnalisés
#[command]
pub fn clear_custom_game_paths(app: tauri::AppHandle) -> Result<(), String> {
    let config_path = get_game_paths_config_file_path(app.path()).map_err(|e| e.to_string())?;

    if config_path.exists() {
        fs::remove_file(config_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}