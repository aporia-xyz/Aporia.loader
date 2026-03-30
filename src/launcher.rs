use anyhow::Result;
use std::env;
use std::fs;
use std::process::Command;

pub struct Launcher {}

impl Launcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn launch(&self, version: &str) -> Result<()> {
        let jre_dir = crate::github::get_jre_dir();
        let versions_dir = crate::github::get_versions_dir();
        let game_dir = crate::github::get_game_dir();

        #[cfg(target_os = "windows")]
        let java_exe = jre_dir.join("jdk-26").join("bin").join("java.exe");

        #[cfg(not(target_os = "windows"))]
        let java_exe = jre_dir.join("jdk-26").join("bin").join("java");

        let jar_path = versions_dir.join("Aporia.client").join("Aporia.client.jar");

        if !java_exe.exists() {
            anyhow::bail!("JRE not found at {:?}", java_exe);
        }

        if !jar_path.exists() {
            anyhow::bail!("Version {} not found at {:?}", version, jar_path);
        }

        log::info!("Launching Minecraft with version {}", version);
        log::info!("Java executable: {:?}", java_exe);
        log::info!("JAR path: {:?}", jar_path);
        log::info!("Game directory: {:?}", game_dir);

        let username = env::var("USERNAME").unwrap_or_else(|_| "Player".to_string());
        let aporia_assets_dir = crate::github::get_aporia_dir().join("assets");

        fs::create_dir_all(&game_dir)?;

        let asset_directory =
            if aporia_assets_dir.exists() && aporia_assets_dir.join("indexes").exists() {
                aporia_assets_dir.clone()
            } else {
                let minecraft_dir = if cfg!(target_os = "windows") {
                    let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
                    std::path::PathBuf::from(appdata)
                        .join(".minecraft")
                        .join("assets")
                } else if cfg!(target_os = "macos") {
                    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    std::path::PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join("minecraft")
                        .join("assets")
                } else {
                    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    std::path::PathBuf::from(home)
                        .join(".minecraft")
                        .join("assets")
                };
                minecraft_dir
            };

        log::info!("Using assets directory: {:?}", asset_directory);

        let mut cmd = Command::new(&java_exe);
        cmd.current_dir(&game_dir);
        cmd.arg("--enable-native-access=ALL-UNNAMED")
            .arg("-Xmx2G")
            .arg("-Xms1G")
            .arg("-cp")
            .arg(&jar_path)
            .arg("net.minecraft.client.main.Main")
            .arg("--version")
            .arg(version)
            .arg("--accessToken")
            .arg("0")
            .arg("--assetsDir")
            .arg(&asset_directory)
            .arg("--assetIndex")
            .arg("29")
            .arg("--userProperties")
            .arg("{}")
            .arg("--gameDir")
            .arg(&game_dir)
            .arg("--username")
            .arg(&username)
            .env("assetDirectory", &asset_directory);

        log::info!("Command: {:?}", cmd);

        let child = cmd.spawn()?;
        log::info!("Minecraft launched with PID: {}", child.id());

        Ok(())
    }
}
