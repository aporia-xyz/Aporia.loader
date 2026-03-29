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
    /// Загрузить файл по URL с прогрессом и retry
    pub async fn download(url: &str, output: &str) -> anyhow::Result<u64> {
        let max_retries = 3;
        let mut attempt = 0;
        
        loop {
            attempt += 1;
            log::info!("Download attempt {}/{} from: {}", attempt, max_retries, url);
            
            match Self::download_internal(url, output).await {
                Ok(size) => {
                    log::info!("Download successful: {} bytes", size);
                    return Ok(size);
                }
                Err(e) => {
                    log::error!("Download attempt {} failed: {}", attempt, e);
                    
                    if attempt >= max_retries {
                        return Err(e);
                    }
                    
                    // Удаляем частичный файл перед retry
                    let _ = tokio::fs::remove_file(output).await;
                    
                    // Ждём перед retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    /// Внутренняя функция загрузки
    async fn download_internal(url: &str, output: &str) -> anyhow::Result<u64> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(300)) // 5 минут таймаут
            .build()
            .context("Failed to create HTTP client")?;

        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        log::info!("Total size: {} bytes ({:.1}MB)", total_size, total_size as f64 / 1_000_000.0);
        
        // Создаем директорию если нужно
        if let Some(parent) = Path::new(output).parent() {
            fs::create_dir_all(parent).context("Failed to create directory")?;
        }

        log::info!("Creating file: {}", output);
        let mut file = tokio::fs::File::create(output)
            .await
            .context("Failed to create file")?;
        
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut chunk_count = 0;
        let mut last_log = std::time::Instant::now();
        
        use futures::stream::StreamExt;
        
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    chunk_count += 1;
                    let chunk_size = chunk.len() as u64;
                    
                    match file.write_all(&chunk).await {
                        Ok(_) => {
                            downloaded += chunk_size;
                            
                            // Логируем прогресс каждую секунду
                            if last_log.elapsed().as_secs() >= 1 {
                                if total_size > 0 {
                                    let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
                                    log::info!("Progress: {}% ({:.1}/{:.1}MB)", 
                                        percent, 
                                        downloaded as f64 / 1_000_000.0, 
                                        total_size as f64 / 1_000_000.0);
                                }
                                last_log = std::time::Instant::now();
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to write chunk {}: {}", chunk_count, e);
                            return Err(e.into());
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to read chunk {}: {}", chunk_count, e);
                    return Err(e.into());
                }
            }
        }
        
        // Flush и sync для гарантии записи
        file.flush().await.context("Failed to flush file")?;
        file.sync_all().await.context("Failed to sync file")?;
        drop(file);
        
        log::info!("Download complete: {} bytes in {} chunks", downloaded, chunk_count);
        
        // Проверяем что файл действительно создан
        let metadata = tokio::fs::metadata(output).await.context("Failed to get file metadata")?;
        log::info!("File size on disk: {:.1}MB", metadata.len() as f64 / 1_000_000.0);
        
        if metadata.len() == 0 {
            anyhow::bail!("Downloaded file is empty!");
        }
        
        if metadata.len() < downloaded {
            anyhow::bail!("File size mismatch: expected {}, got {}", downloaded, metadata.len());
        }
        
        Ok(downloaded)
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
