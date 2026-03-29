//! Утилиты для лаунчера

use std::path::PathBuf;
use std::process::Command;

/// Цвета для консоли (ANSI)
pub mod color {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const BOLD: &str = "\x1b[1m";
}

/// Получить путь по умолчанию для установки
pub fn get_default_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("apr");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library/Application Support/apr");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join(".apr");
        }
    }

    PathBuf::from("./apr")
}

/// Получить имя ОС для Minecraft
pub fn get_os_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";

    #[cfg(target_os = "macos")]
    return "osx";

    #[cfg(target_os = "linux")]
    return "linux";

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown";
}

/// Получить нативный префикс для классификатора
pub fn get_native_classifier() -> &'static str {
    #[cfg(target_os = "windows")]
    return "natives-windows";

    #[cfg(target_os = "macos")]
    return "natives-macos";

    #[cfg(target_os = "linux")]
    return "natives-linux";

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "natives-linux";
}

/// Открыть папку в файловом менеджере
pub fn open_folder(path: &std::path::Path) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }

    Ok(())
}

/// Конвертирует Maven координаты в URL
pub fn maven_to_url(maven: &str, classifier: &str, base_url: &str) -> Option<String> {
    let clean_maven = maven.to_string();

    let parts: Vec<&str> = clean_maven.split(':').collect();
    if parts.len() < 3 {
        return None;
    }

    let group = parts[0];
    let artifact = parts[1];
    let version = if parts.len() > 3 {
        parts[2..parts.len() - 1].join(":") + ":" + parts.last().unwrap()
    } else {
        parts[2].to_string()
    };

    let group_path = group.replace('.', "/");
    let mut filename = format!("{}-{}", artifact, version);
    if !classifier.is_empty() {
        filename.push_str(&format!("-{}", classifier));
    }
    filename.push_str(".jar");

    Some(format!(
        "{}{}/{}/{}/{}",
        base_url, group_path, artifact, version, filename
    ))
}

/// Конвертирует Maven координаты в локальный путь
pub fn maven_to_path(maven: &str, classifier: &str, base_path: &str) -> Option<String> {
    let parts: Vec<&str> = maven.split(':').collect();
    if parts.len() < 3 {
        return None;
    }

    let group = parts[0];
    let artifact = parts[1];
    let version = parts[2];

    let group_path = group.replace('.', "/");
    let mut filename = format!("{}-{}", artifact, version);
    if !classifier.is_empty() {
        filename.push_str(&format!("-{}", classifier));
    }
    filename.push_str(".jar");

    Some(format!(
        "{}/{}/{}/{}/{}",
        base_path, group_path, artifact, version, filename
    ))
}
