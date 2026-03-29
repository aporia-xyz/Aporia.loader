//! Модуль загрузки файлов

use std::path::Path;
use std::fs;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::Context;
use futures::future::join_all;
use tokio::io::AsyncWriteExt;

/// Информация о моде
#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub url: String,
    pub selected: bool,
}

/// Задача на загрузку
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub output: String,
}

/// Статистика загрузки
#[derive(Debug, Default)]
pub struct DownloadStats {
    pub completed: AtomicUsize,
    pub failed: AtomicUsize,
    pub skipped: AtomicUsize,
    pub bytes_downloaded: AtomicU64,
}

/// Загрузчик файлов
pub struct Downloader;

impl Downloader {
    /// Загрузить файл по URL (blocking)
    pub async fn download(url: &str, output: &str) -> anyhow::Result<u64> {
        log::info!("=== DOWNLOAD START ===");
        log::info!("URL: {}", url);
        log::info!("Output: {}", output);
        
        // Создаем директорию если нужно
        let output_path = std::path::Path::new(output);
        if let Some(parent) = output_path.parent() {
            log::info!("Creating dir: {}", parent.display());
            fs::create_dir_all(parent).context("Failed to create directory")?;
        }

        // Используем blocking клиент
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(600))
            .build()
            .context("Failed to create client")?;

        log::info!("Sending request...");
        let mut response = client
            .get(url)
            .send()
            .context("Failed to send request")?;

        log::info!("Response status: {}", response.status());
        
        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        log::info!("Reading response...");
        let mut buffer = Vec::new();
        response.copy_to(&mut buffer).context("Failed to read response")?;
        
        log::info!("Got {} bytes", buffer.len());
        
        log::info!("Writing to: {}", output);
        fs::write(output, &buffer).context("Failed to write file")?;
        
        log::info!("File written: {} bytes", buffer.len());
        log::info!("=== DOWNLOAD COMPLETE ===");
        
        Ok(buffer.len() as u64)
    }

    /// Загрузить файлы параллельно
    pub async fn download_parallel(
        tasks: Vec<DownloadTask>,
        max_concurrent: usize,
        stats: Arc<DownloadStats>,
    ) {
        // Разбиваем на батчи для ограничения параллелизма
        let mut handles = Vec::new();
        
        for task in tasks {
            let stats = Arc::clone(&stats);
            
            // Проверяем существует ли файл
            if Path::new(&task.output).exists() {
                stats.skipped.fetch_add(1, Ordering::SeqCst);
                stats.completed.fetch_add(1, Ordering::SeqCst);
                continue;
            }
            
            let handle = tokio::spawn(async move {
                match Self::download(&task.url, &task.output).await {
                    Ok(bytes) => {
                        stats.bytes_downloaded.fetch_add(bytes, Ordering::SeqCst);
                        stats.completed.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    }
                    Err(_) => {
                        // Удаляем частичный файл
                        let _ = fs::remove_file(&task.output);
                        stats.failed.fetch_add(1, Ordering::SeqCst);
                        stats.completed.fetch_add(1, Ordering::SeqCst);
                        Err(())
                    }
                }
            });
            
            handles.push(handle);
            
            // Ограничиваем количество одновременных задач
            if handles.len() >= max_concurrent {
                let batch: Vec<_> = handles.drain(..).collect();
                join_all(batch).await;
            }
        }
        
        // Обрабатываем оставшиеся
        if !handles.is_empty() {
            join_all(handles).await;
        }
    }

    /// Загрузить моды
    pub async fn download_mods(mods_path: &str, mods: &[Mod]) -> Vec<(String, bool)> {
        let mut results = Vec::new();
        
        if let Err(e) = fs::create_dir_all(mods_path) {
            log::error!("Failed to create mods directory: {}", e);
            return results;
        }

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .danger_accept_invalid_certs(true)
            .build();

        let client = match client {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to create HTTP client: {}", e);
                return results;
            }
        };

        let mut handles = Vec::new();

        for mod_info in mods {
            if !mod_info.selected {
                continue;
            }

            // Извлекаем имя файла из URL
            let filename = mod_info.url.split('/').last().unwrap_or("unknown.jar");
            
            // URL decode
            let decoded_filename = urlencoding::decode(filename)
                .unwrap_or(std::borrow::Cow::Borrowed(filename))
                .to_string();
            
            let output_path = Path::new(mods_path).join(&decoded_filename);
            let url = mod_info.url.clone();
            let name = mod_info.name.clone();
            let client = client.clone();

            let handle = tokio::spawn(async move {
                if output_path.exists() {
                    return (name, true); // Уже загружен
                }

                match client.get(&url).send().await {
                    Ok(response) => {
                        if let Ok(bytes) = response.bytes().await {
                            if let Ok(mut file) = tokio::fs::File::create(&output_path).await {
                                use tokio::io::AsyncWriteExt;
                                let _ = file.write_all(&bytes).await;
                                return (name, true);
                            }
                        }
                    }
                    Err(_) => {}
                }
                (name, false)
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Ok((name, success)) = handle.await {
                results.push((name, success));
            }
        }

        results
    }

    /// Проверить существование файла
    pub fn file_exists(path: &str) -> bool {
        Path::new(path).exists()
    }
}
