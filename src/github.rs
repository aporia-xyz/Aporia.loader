use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub body: String,
    pub is_latest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub short_sha: String,
}

pub struct GitHubClient {
    client: octocrab::Octocrab,
}

impl GitHubClient {
    pub fn new() -> Result<Self> {
        let client = octocrab::Octocrab::builder().build()?;
        Ok(Self { client })
    }

    pub async fn get_releases(&self) -> Result<Vec<Release>> {
        let releases = self
            .client
            .repos("dakychan", "Aporia")
            .releases()
            .list()
            .send()
            .await?;

        let mut result = Vec::new();
        for (idx, release) in releases.items.iter().enumerate() {
            result.push(Release {
                tag_name: release.tag_name.clone(),
                name: release.name.clone().unwrap_or_default(),
                published_at: release.published_at.map(|d| d.to_string()).unwrap_or_default(),
                body: release.body.clone().unwrap_or_default(),
                is_latest: idx == 0,
            });
        }

        Ok(result)
    }

    pub async fn get_commits(&self, branch: &str) -> Result<Vec<Commit>> {
        let commits = self
            .client
            .repos("dakychan", "Aporia")
            .list_commits()
            .branch(branch)
            .per_page(10)
            .send()
            .await?;

        let mut result = Vec::new();
        for commit in commits.items {
            let sha = commit.sha.clone();
            let short_sha = sha.chars().take(7).collect();
            let message = commit
                .commit
                .message
                .lines()
                .next()
                .unwrap_or("")
                .to_string();

            result.push(Commit {
                sha,
                short_sha,
                message,
            });
        }

        Ok(result)
    }
}

pub fn get_aporia_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("apr")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("apr")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".apr")
    }
}

pub fn get_game_dir() -> PathBuf {
    get_aporia_dir().join("game")
}

pub fn get_jre_dir() -> PathBuf {
    get_aporia_dir().join("jre")
}

pub fn get_versions_dir() -> PathBuf {
    get_aporia_dir().join("versions")
}

pub fn get_config_path() -> PathBuf {
    get_aporia_dir().join("config.apr")
}
