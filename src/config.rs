//! Модуль конфигурации лаунчера

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader, Write};

use crate::utils;

/// Конфигурация лаунчера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Путь установки
    pub install_path: String,
    /// Выделено RAM (MB)
    pub ram_mb: u32,
    /// Имя пользователя
    pub username: String,
    /// Режим разработчика (-noverify)
    pub dev_mode: bool,
    /// Путь к Java
    pub java_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            install_path: utils::get_default_path().to_string_lossy().to_string(),
            ram_mb: 4096,
            username: "Player".to_string(),
            dev_mode: false,
            java_path: String::new(),
        }
    }
}

impl Config {
    /// Создать новую конфигурацию с путем к файлу
    pub fn new(config_path: &Path) -> Self {
        let mut config = Self::default();
        config.load(config_path);
        config
    }
    
    /// Загрузить конфигурацию из файла
    pub fn load(&mut self, config_path: &Path) {
        if !config_path.exists() {
            return;
        }
        
        let file = match fs::File::open(config_path) {
            Ok(f) => f,
            Err(_) => return,
        };
        
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                
                match key.as_str() {
                    "path" => self.install_path = value,
                    "ram" => {
                        if let Ok(ram) = value.parse() {
                            self.ram_mb = ram;
                        }
                    }
                    "username" => self.username = value,
                    "devmode" => self.dev_mode = value == "true",
                    "javapath" => self.java_path = value,
                    _ => {}
                }
            }
        }
    }
    
    /// Сохранить конфигурацию в файл
    pub fn save(&self, config_path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = fs::File::create(config_path)?;
        writeln!(file, "path={}", self.install_path)?;
        writeln!(file, "ram={}", self.ram_mb)?;
        writeln!(file, "username={}", self.username)?;
        writeln!(file, "devmode={}", if self.dev_mode { "true" } else { "false" })?;
        if !self.java_path.is_empty() {
            writeln!(file, "javapath={}", self.java_path)?;
        }
        
        Ok(())
    }
    
    /// Получить путь к файлу конфигурации
    pub fn config_path(&self) -> PathBuf {
        PathBuf::from(&self.install_path).join("config.txt")
    }
}
