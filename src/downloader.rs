use anyhow::Result;
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        log::info!("Downloading {} to {:?}", url, dest);

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;

        let mut file = File::create(dest).await?;
        file.write_all(&bytes).await?;

        log::info!("Downloaded {} successfully", url);
        Ok(())
    }

    pub async fn download_version(&self, version: &str) -> Result<()> {
        let base_url = format!("https://github.com/dakychan/Aporia/releases/download/{}", version);
        let versions_dir = crate::github::get_versions_dir().join("Aporia.client");

        tokio::fs::create_dir_all(&versions_dir).await?;

        // Download jar
        let jar_url = format!("{}/Aporia.client.jar", base_url);
        let jar_path = versions_dir.join("Aporia.client.jar");
        self.download_file(&jar_url, &jar_path).await?;

        // Download json
        let json_url = format!("{}/Aporia.client.json", base_url);
        let json_path = versions_dir.join("Aporia.client.json");
        self.download_file(&json_url, &json_path).await?;

        Ok(())
    }

    pub async fn download_jre(&self) -> Result<()> {
        let jre_dir = crate::github::get_jre_dir();
        
        // Check if JRE already exists
        if jre_dir.exists() {
            log::info!("JRE already exists, skipping download");
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        let jre_url = "https://download.java.net/java/GA/jdk26/c3cc523845074aa0af4f5e1e1ed4151d/35/GPL/openjdk-26_windows-x64_bin.zip";
        
        #[cfg(target_os = "macos")]
        let jre_url = "https://download.java.net/java/GA/jdk26/c3cc523845074aa0af4f5e1e1ed4151d/35/GPL/openjdk-26_macos-x64_bin.tar.gz";
        
        #[cfg(target_os = "linux")]
        let jre_url = "https://download.java.net/java/GA/jdk26/c3cc523845074aa0af4f5e1e1ed4151d/35/GPL/openjdk-26_linux-x64_bin.tar.gz";

        let aporia_dir = crate::github::get_aporia_dir();
        tokio::fs::create_dir_all(&aporia_dir).await?;

        let archive_path = aporia_dir.join("jre_temp.zip");
        self.download_file(jre_url, &archive_path).await?;

        log::info!("Extracting JRE...");
        
        // Extract archive
        let file = std::fs::File::open(&archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(&jre_dir)?;

        // Remove archive
        tokio::fs::remove_file(&archive_path).await?;

        log::info!("JRE installed successfully");
        Ok(())
    }
}
