use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::command;

#[cfg(target_os = "windows")]
use std::env;

#[cfg(any(target_os = "windows", target_os = "linux"))]
use tokio::process::Command;

#[derive(Serialize)]
struct FolderInfo {
    name: String,
    weight: String,
    path: String,
}

#[derive(Serialize)]
struct Output {
    folders: Vec<FolderInfo>,
}

#[cfg(target_os = "windows")]
fn get_cache_path() -> String {
    let appdata = env::var("LOCALAPPDATA")
        .expect("Impossible de lire la variable d'environnement LOCALAPPDATA");
    format!("{}\\Star Citizen", appdata)
}

fn get_weight(path: &Path) -> String {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Impossible de lire le répertoire") {
            let entry = entry.expect("Erreur lors de la lecture de l'entrée");
            let path = entry.path();
            if path.is_file() {
                total_size += fs::metadata(&path)
                    .expect("Impossible de lire les métadonnées")
                    .len();
            } else if path.is_dir() {
                total_size += get_folder_size(&path);
            }
        }
    }

    if total_size < 1_048_576 {
        let size_in_kb = total_size as f64 / 1_024.0;
        format!("{:.0} Ko", size_in_kb)
    } else {
        let size_in_megabytes = total_size as f64 / 1_048_576.0;
        format!("{:.1} Mo", size_in_megabytes)
    }
}

fn get_folder_size(path: &Path) -> u64 {
    let mut total_size = 0;

    for entry in fs::read_dir(path).expect("Impossible de lire le répertoire") {
        let entry = entry.expect("Erreur lors de la lecture de l'entrée");
        let path = entry.path();
        if path.is_file() {
            total_size += fs::metadata(&path)
                .expect("Impossible de lire les métadonnées")
                .len();
        } else if path.is_dir() {
            total_size += get_folder_size(&path);
        }
    }

    total_size
}

fn get_folder_info(path: &Path) -> FolderInfo {
    let folder_name = path.file_name().unwrap().to_string_lossy().to_string();
    let folder_weight = get_weight(path);
    let folder_path = path.to_string_lossy().to_string();

    FolderInfo {
        name: folder_name,
        weight: folder_weight,
        path: folder_path,
    }
}

// ─── Helpers Linux ────────────────────────────────────────────────────────────

/// Depuis un chemin d'installation Star Citizen sous Proton/Wine, dérive le ou les
/// chemins de cache. Cherche d'abord `drive_c/users/*/AppData/Local/Star Citizen/`
/// (cache RSI Launcher), puis utilise le dossier `user/` du jeu en fallback.
#[cfg(target_os = "linux")]
fn derive_linux_cache_paths(game_install_path: &str) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let path = Path::new(game_install_path);
    let mut accumulated = PathBuf::new();
    let mut drive_c_root: Option<PathBuf> = None;

    for component in path.components() {
        accumulated.push(component);
        if component.as_os_str() == "drive_c" {
            drive_c_root = Some(accumulated.clone());
            break;
        }
    }

    let mut cache_paths = Vec::new();

    // 1. Chercher le cache RSI Launcher dans drive_c/users/*/AppData/Local/Star Citizen/
    if let Some(drive_c) = drive_c_root {
        let users_dir = drive_c.join("users");
        if let Ok(entries) = fs::read_dir(&users_dir) {
            for entry in entries.flatten() {
                let candidate = entry
                    .path()
                    .join("AppData")
                    .join("Local")
                    .join("Star Citizen");
                if candidate.is_dir() {
                    cache_paths.push(candidate);
                }
            }
        }
    }

    // 2. Fallback : dossier user/ à l'intérieur du répertoire du jeu
    if cache_paths.is_empty() {
        let game_user_dir = path.join("user");
        if game_user_dir.is_dir() {
            cache_paths.push(game_user_dir);
        }
    }

    cache_paths
}

/// Charge tous les chemins de jeu sauvegardés et retourne la liste de tous les
/// dossiers de cache Wine correspondants, dédupliqués.
#[cfg(target_os = "linux")]
fn get_all_linux_cache_paths(app: &tauri::AppHandle) -> Vec<std::path::PathBuf> {
    use tauri::Manager;

    let config_dir = match app.path().app_config_dir() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let config_file = config_dir.join("game_paths.json");
    if !config_file.exists() {
        return Vec::new();
    }

    let json_data = match fs::read_to_string(&config_file) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    #[derive(serde::Deserialize)]
    struct GamePaths {
        paths: Vec<String>,
    }

    let game_paths: GamePaths = match serde_json::from_str(&json_data) {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    let mut all_cache_paths: Vec<std::path::PathBuf> = Vec::new();
    for game_path in &game_paths.paths {
        for cache_path in derive_linux_cache_paths(game_path) {
            if !all_cache_paths.contains(&cache_path) {
                all_cache_paths.push(cache_path);
            }
        }
    }

    all_cache_paths
}

// ─── Commandes Tauri ──────────────────────────────────────────────────────────

/// Récupère les informations sur les dossiers de cache de Star Citizen.
///
/// Retourne un JSON contenant la liste des dossiers avec leur nom, taille et chemin.
#[command]
pub fn get_cache_informations(app: tauri::AppHandle) -> String {
    #[cfg(target_os = "windows")]
    {
        let star_citizen_path = get_cache_path();
        let mut folders = Vec::new();

        match fs::read_dir(&star_citizen_path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.expect("Erreur lors de la lecture de l'entrée");
                    let path = entry.path();
                    if path.is_dir() {
                        folders.push(get_folder_info(&path));
                    }
                }
            }
            Err(e) => {
                println!(
                    "Erreur lors de l'accès au répertoire {}: {}",
                    star_citizen_path, e
                );
            }
        }

        let output = Output { folders };
        return serde_json::to_string_pretty(&output)
            .expect("Erreur lors de la sérialisation en JSON");
    }

    #[cfg(target_os = "linux")]
    {
        let cache_paths = get_all_linux_cache_paths(&app);

        if cache_paths.is_empty() {
            return r#"{"folders":[]}"#.to_string();
        }

        let mut folders = Vec::new();

        for cache_root in &cache_paths {
            if let Ok(entries) = fs::read_dir(cache_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        folders.push(get_folder_info(&path));
                    }
                }
            }
        }

        let output = Output { folders };
        serde_json::to_string_pretty(&output)
            .unwrap_or_else(|_| r#"{"folders":[]}"#.to_string())
    }
}

/// Supprime un dossier ou un fichier au chemin spécifié.
///
/// Retourne `true` si la suppression a réussi, `false` sinon.
#[command]
pub fn delete_folder(path: &str) -> bool {
    let path = Path::new(path);
    if path.is_dir() {
        match fs::remove_dir_all(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    } else {
        match fs::remove_file(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// Supprime tous les dossiers et fichiers du cache de Star Citizen.
///
/// Retourne `true` si tous les éléments ont été supprimés avec succès, `false` sinon.
#[command]
pub fn clear_cache(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        let star_citizen_path = get_cache_path();

        if let Ok(entries) = fs::read_dir(&star_citizen_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Err(_) = fs::remove_dir_all(&path) {
                            return false;
                        }
                    } else {
                        if let Err(_) = fs::remove_file(&path) {
                            return false;
                        }
                    }
                }
            }
            true
        } else {
            false
        }
    }

    #[cfg(target_os = "linux")]
    {
        let cache_paths = get_all_linux_cache_paths(&app);

        if cache_paths.is_empty() {
            return false;
        }

        let mut all_ok = true;

        for cache_root in &cache_paths {
            if let Ok(entries) = fs::read_dir(cache_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let ok = if path.is_dir() {
                        fs::remove_dir_all(&path).is_ok()
                    } else {
                        fs::remove_file(&path).is_ok()
                    };
                    if !ok {
                        all_ok = false;
                    }
                }
            } else {
                all_ok = false;
            }
        }

        all_ok
    }
}

/// Ouvre le dossier de cache de Star Citizen dans le gestionnaire de fichiers.
#[command]
pub async fn open_cache_folder(app: tauri::AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let star_citizen_path = get_cache_path();

        if std::path::Path::new(&star_citizen_path).exists() {
            Command::new("explorer")
                .arg(&star_citizen_path)
                .spawn()
                .map_err(|e| format!("Erreur lors de l'ouverture du dossier : {}", e))?;
            Ok(true)
        } else {
            Err(format!("Le dossier '{}' n'existe pas.", star_citizen_path))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let first_path = get_all_linux_cache_paths(&app)
            .into_iter()
            .find(|p| p.exists());

        match first_path {
            Some(path) => {
                Command::new("xdg-open")
                    .arg(path.to_string_lossy().as_ref())
                    .spawn()
                    .map_err(|e| format!("Erreur lors de l'ouverture du dossier : {}", e))?;
                Ok(true)
            }
            None => Err(
                "Aucun dossier de cache Star Citizen trouvé. Vérifiez vos chemins de jeu dans les préférences.".to_string(),
            ),
        }
    }
}
