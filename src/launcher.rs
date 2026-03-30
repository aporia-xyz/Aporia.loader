use anyhow::Result;
use std::env;
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
        let java_exe = jre_dir.join("jdk-26").join("bin").join("javaw.exe");

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

        let username = env::var("USERNAME").unwrap_or_else(|_| "Player".to_string());
        let assets_dir = game_dir.join("assets");

        let mut cmd = Command::new(&java_exe);
        cmd.arg("-Xmx2G")
            .arg("-Xms1G")
            .arg("-cp")
            .arg(&jar_path)
            .arg("mcp.client.Start")
            .arg("--version")
            .arg(version)
            .arg("--accessToken")
            .arg("0")
            .arg("--assetsDir")
            .arg(&assets_dir)
            .arg("--assetIndex")
            .arg("29")
            .arg("--userProperties")
            .arg("{}")
            .arg("--gameDir")
            .arg(&game_dir)
            .arg("--username")
            .arg(&username);

        log::info!("Command: {:?}", cmd);

        let child = cmd.spawn()?;
        log::info!("Minecraft launched with PID: {}", child.id());

        Ok(())
    }
}
