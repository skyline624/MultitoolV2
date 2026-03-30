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

// ─── Helpers communs ──────────────────────────────────────────────────────────

/// Calcule récursivement la taille d'un dossier en octets.
/// Les entrées illisibles (permissions, liens brisés) sont silencieusement ignorées.
fn get_folder_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| {
            let p = e.path();
            if p.is_file() {
                fs::metadata(&p).map(|m| m.len()).unwrap_or(0)
            } else if p.is_dir() {
                get_folder_size(&p)
            } else {
                0
            }
        })
        .sum()
}

/// Formate la taille d'un dossier en Ko ou Mo.
fn get_weight(path: &Path) -> String {
    let total = get_folder_size(path);
    if total < 1_048_576 {
        format!("{:.0} Ko", total as f64 / 1_024.0)
    } else {
        format!("{:.1} Mo", total as f64 / 1_048_576.0)
    }
}

fn get_folder_info(path: &Path) -> FolderInfo {
    FolderInfo {
        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        weight: get_weight(path),
        path: path.to_string_lossy().to_string(),
    }
}

// ─── Helpers Windows ──────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn get_cache_path() -> Result<String, String> {
    let appdata = env::var("LOCALAPPDATA")
        .map_err(|_| "Variable d'environnement LOCALAPPDATA non définie".to_string())?;
    Ok(format!("{}\\Star Citizen", appdata))
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

/// Dérive les chemins de cache pour toutes les installations détectées automatiquement.
#[cfg(target_os = "linux")]
fn get_all_linux_cache_paths() -> Vec<std::path::PathBuf> {
    use crate::scripts::gamepath::get_star_citizen_versions;

    let mut all: Vec<std::path::PathBuf> = Vec::new();
    for info in get_star_citizen_versions().versions.into_values() {
        for cache_path in derive_linux_cache_paths(&info.path) {
            if !all.contains(&cache_path) {
                all.push(cache_path);
            }
        }
    }
    all
}

// ─── Commandes Tauri ──────────────────────────────────────────────────────────

/// Récupère les informations sur les dossiers de cache de Star Citizen.
///
/// Retourne un JSON contenant la liste des dossiers avec leur nom, taille et chemin.
#[command]
pub fn get_cache_informations() -> String {
    #[cfg(target_os = "windows")]
    {
        let star_citizen_path = match get_cache_path() {
            Ok(p) => p,
            Err(_) => return r#"{"folders":[]}"#.to_string(),
        };
        let mut folders = Vec::new();

        if let Ok(entries) = fs::read_dir(&star_citizen_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    folders.push(get_folder_info(&path));
                }
            }
        }

        let output = Output { folders };
        return serde_json::to_string_pretty(&output)
            .unwrap_or_else(|_| r#"{"folders":[]}"#.to_string());
    }

    #[cfg(target_os = "linux")]
    {
        let cache_paths = get_all_linux_cache_paths();

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
        fs::remove_dir_all(path).is_ok()
    } else {
        fs::remove_file(path).is_ok()
    }
}

/// Supprime tous les dossiers et fichiers du cache de Star Citizen.
///
/// Retourne `true` si tous les éléments ont été supprimés avec succès, `false` sinon.
#[command]
pub fn clear_cache() -> bool {
    #[cfg(target_os = "windows")]
    {
        let star_citizen_path = match get_cache_path() {
            Ok(p) => p,
            Err(_) => return false,
        };

        if let Ok(entries) = fs::read_dir(&star_citizen_path) {
            let mut all_ok = true;
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
            return all_ok;
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        let cache_paths = get_all_linux_cache_paths();

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
pub async fn open_cache_folder() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let star_citizen_path = get_cache_path()?;

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
        let first_path = get_all_linux_cache_paths()
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
