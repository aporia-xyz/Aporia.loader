use anyhow::Result;
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use serde_json::json;

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

    pub async fn download_assets(&self, version: &str) -> Result<()> {
        let versions_dir = crate::github::get_versions_dir();
        let json_path = versions_dir.join("Aporia.client").join("Aporia.client.json");
        
        if !json_path.exists() {
            log::warn!("Version JSON not found, skipping assets download");
            return Ok(());
        }

        let json_content = tokio::fs::read_to_string(&json_path).await?;
        let version_json: serde_json::Value = serde_json::from_str(&json_content)?;

        if !version_json.get("assetIndex").is_some() {
            log::warn!("assetIndex not found in version JSON");
            return Ok(());
        }

        let asset_index = &version_json["assetIndex"];
        let asset_index_url = asset_index["url"].as_str().unwrap_or("");
        let asset_index_id = asset_index["id"].as_str().unwrap_or("29");

        let assets_dir = crate::github::get_aporia_dir().join("assets");
        let index_path = assets_dir.join("indexes").join(format!("{}.json", asset_index_id));

        // Download asset index
        tokio::fs::create_dir_all(index_path.parent().unwrap()).await?;
        if !index_path.exists() {
            log::info!("Downloading asset index...");
            self.download_file(asset_index_url, &index_path).await?;
        }

        // Parse asset index and download all assets
        let index_content = tokio::fs::read_to_string(&index_path).await?;
        let index_json: serde_json::Value = serde_json::from_str(&index_content)?;

        if let Some(objects) = index_json.get("objects").and_then(|o| o.as_object()) {
            let total = objects.len();
            let mut current = 0;

            log::info!("Downloading {} assets...", total);

            for (_, asset_data) in objects.iter() {
                current += 1;
                
                if let Some(hash) = asset_data.get("hash").and_then(|h| h.as_str()) {
                    let subdir = &hash[0..2];
                    let asset_path = assets_dir.join("objects").join(subdir).join(hash);

                    if !asset_path.exists() {
                        let url = format!("https://resources.download.minecraft.net/{}/{}", subdir, hash);
                        if let Err(e) = self.download_file(&url, &asset_path).await {
                            log::warn!("Failed to download asset {}: {}", hash, e);
                        }
                    }

                    if current % 100 == 0 || current == total {
                        let percent = (current * 100) / total;
                        log::info!("Assets: {}/{} ({}%)", current, total, percent);
                    }
                }
            }

            log::info!("Assets download completed");
        }

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
