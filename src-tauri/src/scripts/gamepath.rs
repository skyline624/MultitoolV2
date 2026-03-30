use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use tauri::command;

fn get_log_file_path() -> Option<String> {
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            let rsi_launcher_path = format!("{}\\rsilauncher", appdata);
            let log_file_path = format!("{}\\logs\\log.log", rsi_launcher_path);
            return Some(log_file_path);
        }
    }
    None
}

fn get_launcher_log_list() -> Vec<String> {
    if let Some(log_file_path) = get_log_file_path() {
        if let Ok(contents) = fs::read_to_string(log_file_path) {
            return contents.lines().map(|s| s.to_string()).collect();
        }
    }
    Vec::new()
}

#[cfg(target_os = "windows")]
fn check_and_add_path(path: &str, check_exists: bool, sc_install_paths: &mut Vec<String>) {
    let path = path.replace("\\\\", "\\");
    let normalized_path = path.trim_end_matches('\\').to_string();

    if !normalized_path.is_empty()
        && !sc_install_paths
            .iter()
            .any(|existing_path| existing_path.trim_end_matches('\\') == normalized_path)
    {
        if !check_exists {
            sc_install_paths.push(normalized_path);
        } else {
            let exe_path = format!("{}\\Bin64\\StarCitizen.exe", normalized_path);
            let data_p4k_path = format!("{}\\Data.p4k", normalized_path);
            if Path::new(&exe_path).exists() && Path::new(&data_p4k_path).exists() {
                sc_install_paths.push(normalized_path);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn check_and_add_path(path: &str, check_exists: bool, sc_install_paths: &mut Vec<String>) {
    let normalized_path = path.trim_end_matches('/').trim_end_matches('\\').to_string();

    if !normalized_path.is_empty()
        && !sc_install_paths
            .iter()
            .any(|existing_path| existing_path.trim_end_matches('/') == normalized_path.trim_end_matches('/'))
    {
        if !check_exists {
            sc_install_paths.push(normalized_path);
        } else {
            // Sur Linux/Proton, vérifier les fichiers dans le préfixe Wine
            let exe_path = format!("{}/Bin64/StarCitizen.exe", normalized_path);
            let data_p4k_path = format!("{}/Data.p4k", normalized_path);
            if Path::new(&exe_path).exists() && Path::new(&data_p4k_path).exists() {
                sc_install_paths.push(normalized_path);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn get_game_install_path(list_data: Vec<String>, check_exists: bool) -> Vec<String> {
    let mut sc_install_paths = Vec::new();

    let pattern = r"([a-zA-Z]:\\\\(?:[^\\\\]+\\\\)*StarCitizen\\\\[A-Za-z0-9_\\.\\@\\-]+)";
    let re = Regex::new(pattern).unwrap();

    for line in list_data.iter().rev() {
        for cap in re.captures_iter(line) {
            if let Some(matched_path) = cap.get(0) {
                check_and_add_path(matched_path.as_str(), check_exists, &mut sc_install_paths);
            }
        }
    }

    sc_install_paths
}

#[cfg(target_os = "linux")]
fn get_game_install_path(_list_data: Vec<String>, check_exists: bool) -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let search_roots = [
        format!("{}/.steam/steam/steamapps/common", home),
        format!("{}/.local/share/Steam/steamapps/common", home),
        format!("{}/.var/app/com.valvesoftware.Steam/.steam/steam/steamapps/common", home),
        format!("{}/Games", home),
        format!("{}/games", home),
    ];

    let mut found = Vec::new();

    for root in &search_roots {
        let sc_dir = std::path::Path::new(root).join("StarCitizen");
        if !sc_dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&sc_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    check_and_add_path(&path.to_string_lossy(), check_exists, &mut found);
                }
            }
        }
    }

    found
}

#[cfg(target_os = "windows")]
fn get_game_channel_id(install_path: &str) -> String {
    let re = Regex::new(r"StarCitizen\\([A-Za-z0-9_\\.\\@-]+)\\?$").unwrap();
    if let Some(cap) = re.captures(install_path) {
        if let Some(version) = cap.get(1) {
            let version_str = version.as_str();
            return version_str.trim_end_matches('\\').to_string();
        }
    }
    "UNKNOWN".to_string()
}

#[cfg(target_os = "linux")]
fn get_game_channel_id(install_path: &str) -> String {
    // Sur Linux, extraire le canal du chemin (format Windows ou Unix)
    let re_unix = Regex::new(r"StarCitizen/([A-Za-z0-9_\\.\\@-]+)/?$").unwrap();
    let re_windows = Regex::new(r"StarCitizen\\([A-Za-z0-9_\\.\\@-]+)\\?$").unwrap();

    if let Some(cap) = re_unix.captures(install_path) {
        if let Some(version) = cap.get(1) {
            return version.as_str().trim_end_matches('/').to_string();
        }
    }

    if let Some(cap) = re_windows.captures(install_path) {
        if let Some(version) = cap.get(1) {
            return version.as_str().trim_end_matches('\\').to_string();
        }
    }

    "UNKNOWN".to_string()
}

/// Informations sur une version installée de Star Citizen.
#[derive(Serialize)]
pub struct VersionInfo {
    pub path: String,
    pub translated: bool,
    pub up_to_date: bool,
}

/// Collection de toutes les versions de Star Citizen détectées.
#[derive(Serialize)]
pub struct VersionPaths {
    pub versions: HashMap<String, VersionInfo>,
}

/// Détecte et retourne toutes les versions installées de Star Citizen.
///
/// Analyse les logs du launcher RSI pour trouver les chemins d'installation.
#[command]
pub fn get_star_citizen_versions() -> VersionPaths {
    let log_lines = get_launcher_log_list();
    let sc_install_paths = get_game_install_path(log_lines, true);

    let mut versions = HashMap::new();
    for path in &sc_install_paths {
        let normalized_path = path.trim_end_matches('\\').trim_end_matches('/').to_string();
        let version = get_game_channel_id(&normalized_path);

        if version != "UNKNOWN" && !versions.contains_key(&version) {
            versions.insert(
                version,
                VersionInfo {
                    path: normalized_path,
                    translated: false,
                    up_to_date: false,
                },
            );
        }
    }
    VersionPaths { versions }
}

