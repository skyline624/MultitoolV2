use tauri::command;

#[cfg(target_os = "windows")]
use auto_launch::AutoLaunch;

#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "linux")]
fn get_autostart_path() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("autostart").join("multitoolv2.desktop"))
}

#[cfg(target_os = "linux")]
fn create_desktop_entry() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Impossible de récupérer le chemin de l'exécutable: {}", e))?;

    let desktop_entry = format!(
        "[Desktop Entry]\n\
Type=Application\n\
Name=MultitoolV2\n\
Exec=\"{}\" --minimized\n\
Hidden=false\n\
NoDisplay=false\n\
X-GNOME-Autostart-enabled=true\n",
        exe_path.display()
    );

    if let Some(path) = get_autostart_path() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Impossible de créer le répertoire autostart: {}", e))?;
        }
        std::fs::write(&path, desktop_entry)
            .map_err(|e| format!("Impossible d'écrire le fichier desktop: {}", e))?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn remove_desktop_entry() -> Result<(), String> {
    if let Some(path) = get_autostart_path() {
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Impossible de supprimer le fichier desktop: {}", e))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn is_desktop_entry_enabled() -> bool {
    get_autostart_path().map(|p| p.exists()).unwrap_or(false)
}

/// Active le démarrage automatique de l'application au démarrage du système.
///
/// L'application sera lancée minimisée dans le system tray.
#[command]
pub fn enable_auto_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let app_name = "MultitoolV2";
        let app_path = std::env::current_exe()
            .map_err(|e| format!("Impossible de récupérer le chemin de l'exécutable: {}", e))?;

        let auto_launch = AutoLaunch::new(
            app_name,
            app_path.to_str().ok_or("Chemin invalide")?,
            &["--minimized"],
        );

        auto_launch.enable().map_err(|e| {
            format!(
                "Erreur lors de l'activation du démarrage automatique: {}",
                e
            )
        })?;

        println!("[Startup Manager] Démarrage automatique activé");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        create_desktop_entry()?;
        println!("[Startup Manager] Démarrage automatique activé (Linux)");
        Ok(())
    }
}

/// Désactive le démarrage automatique de l'application.
#[command]
pub fn disable_auto_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let app_name = "MultitoolV2";
        let app_path = std::env::current_exe()
            .map_err(|e| format!("Impossible de récupérer le chemin de l'exécutable: {}", e))?;

        let auto_launch = AutoLaunch::new(
            app_name,
            app_path.to_str().ok_or("Chemin invalide")?,
            &["--minimized"],
        );

        auto_launch.disable().map_err(|e| {
            format!(
                "Erreur lors de la désactivation du démarrage automatique: {}",
                e
            )
        })?;

        println!("[Startup Manager] Démarrage automatique désactivé");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        remove_desktop_entry()?;
        println!("[Startup Manager] Démarrage automatique désactivé (Linux)");
        Ok(())
    }
}

/// Vérifie si le démarrage automatique est activé.
#[command]
pub fn is_auto_startup_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let app_name = "MultitoolV2";
        let app_path = std::env::current_exe()
            .map_err(|e| format!("Impossible de récupérer le chemin de l'exécutable: {}", e))?;

        let auto_launch = AutoLaunch::new(
            app_name,
            app_path.to_str().ok_or("Chemin invalide")?,
            &["--minimized"],
        );

        auto_launch.is_enabled().map_err(|e| {
            format!(
                "Erreur lors de la vérification du démarrage automatique: {}",
                e
            )
        })
    }

    #[cfg(target_os = "linux")]
    {
        Ok(is_desktop_entry_enabled())
    }
}