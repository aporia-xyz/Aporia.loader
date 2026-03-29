//! Aporia Loader - Cross-platform Minecraft Loader
//! 
//! Кроссплатформенный лаунчер с GUI для Minecraft.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod downloader;
mod utils;

use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::sync::mpsc;

use eframe::egui;
use serde_json::Value as JsonValue;

use config::Config;
use downloader::{Downloader, Mod as ModInfo};

/// Версия лаунчера
const VERSION: &str = "0.3.0";

/// Доступные версии Minecraft
#[derive(Debug, Clone, PartialEq)]
enum McVersion {
    Fabric,
    MCP,
}

impl McVersion {
    fn name(&self) -> &'static str {
        match self {
            McVersion::Fabric => "Fabric 1.21.11 (Modded)",
            McVersion::MCP => "MCP v (last build)",
        }
    }
    
    fn branch(&self) -> &'static str {
        match self {
            McVersion::Fabric => "fabric",
            McVersion::MCP => "mcp",
        }
    }
    
    fn id(&self) -> &'static str {
        match self {
            McVersion::Fabric => "fabric-1.21.11",
            McVersion::MCP => "mcp-latest",
        }
    }
}

/// Состояния приложения
#[derive(PartialEq, Clone)]
enum AppState {
    Login,
    Main,
    Settings,
}

/// Changelog entry
#[derive(Debug, Clone)]
struct ChangelogEntry {
    version: String,
    date: String,
    changes: Vec<String>,
}

/// Главное приложение
struct AporiaApp {
    state: AppState,
    config: Config,
    selected_version: McVersion,
    
    // Login screen
    username_input: String,
    login_animation: f32,
    
    // Main screen
    changelog: Vec<ChangelogEntry>,
    changelog_loading: bool,
    current_changelog_index: usize,
    main_animation: f32,
    changelog_rx: Option<mpsc::Receiver<Vec<ChangelogEntry>>>,
    
    // Commits for versions
    version_commits: Vec<String>,
    commits_rx: Option<mpsc::Receiver<Vec<String>>>,
    
    // Launch status
    is_launching: bool,
    launch_progress: f32,
    launch_message: String,
    launch_complete: bool,
    
    // Settings
    temp_ram: u32,
    temp_dev_mode: bool,
    
    // Async
    rx: Option<mpsc::Receiver<String>>,
    mods: Vec<ModInfo>,
}

impl Default for AporiaApp {
    fn default() -> Self {
        let config = Config::default();
        
        Self {
            state: AppState::Login,
            config: config.clone(),
            selected_version: McVersion::Fabric,
            username_input: config.username.clone(),
            login_animation: 0.0,
            changelog: Vec::new(),
            changelog_loading: false,
            current_changelog_index: 0,
            main_animation: 0.0,
            changelog_rx: None,
            version_commits: Vec::new(),
            commits_rx: None,
            is_launching: false,
            launch_progress: 0.0,
            launch_message: String::new(),
            launch_complete: false,
            temp_ram: config.ram_mb,
            temp_dev_mode: config.dev_mode,
            rx: None,
            mods: vec![
                ModInfo {
                    name: "Mod Menu".to_string(),
                    url: "https://cdn.modrinth.com/data/mOgUt4GM/versions/JWQVh32x/modmenu-17.0.0-beta.2.jar".to_string(),
                    selected: true,
                },
                ModInfo {
                    name: "3D Skin Layers".to_string(),
                    url: "https://cdn.modrinth.com/data/zV5r3pPn/versions/JS9deRtw/skinlayers3d-fabric-1.10.2-mc1.21.11.jar".to_string(),
                    selected: true,
                },
                ModInfo {
                    name: "Sound Physics Remastered".to_string(),
                    url: "https://cdn.modrinth.com/data/qyVF9oeo/versions/pfqxi9qs/sound-physics-remastered-fabric-1.21.11-1.5.1.jar".to_string(),
                    selected: true,
                },
                ModInfo {
                    name: "Cloth Config".to_string(),
                    url: "https://cdn.modrinth.com/data/9s6osm5g/versions/xuX40TN5/cloth-config-21.11.153-fabric.jar".to_string(),
                    selected: true,
                },
            ],
        }
    }
}

impl AporiaApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Настройка стилей - тёмная тема
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 10.0);
        style.visuals.button_frame = true;
        style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(15, 15, 20, 255);
        style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(12, 12, 18, 255);
        style.visuals.extreme_bg_color = egui::Color32::from_rgba_unmultiplied(8, 8, 12, 255);
        
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(25, 25, 35, 200);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(35, 35, 50, 210);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(45, 45, 65, 220);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(55, 55, 80, 230);
        
        cc.egui_ctx.set_style(style);
        
        let mut app = Self::default();
        app.load_changelog();
        app
    }
    
    /// Загрузка коммитов для версии
    fn load_commits(&mut self) {
        let branch = self.selected_version.branch().to_string();
        
        log::info!("Loading commits from branch: {}", branch);
        
        let (tx, rx) = mpsc::channel::<Vec<String>>();
        self.commits_rx = Some(rx);
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                match fetch_commits_for_version(&branch).await {
                    Ok(commits) => {
                        log::info!("Successfully loaded {} commits from {}", commits.len(), branch);
                        for (i, commit) in commits.iter().take(5).enumerate() {
                            log::info!("  {}: {}", i + 1, commit);
                        }
                        commits
                    }
                    Err(e) => {
                        log::error!("Failed to load commits: {}", e);
                        vec!["Failed to load commits".to_string()]
                    }
                }
            });
            let _ = tx.send(result);
        });
    }
    
    /// Загрузка changelog из GitHub
    fn load_changelog(&mut self) {
        self.changelog_loading = true;
        self.changelog = default_changelog();
        
        log::info!("Loading changelog from GitHub");
        
        let (tx, rx) = mpsc::channel::<Vec<ChangelogEntry>>();
        self.changelog_rx = Some(rx);
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                match fetch_aporia_releases().await {
                    Ok(entries) => {
                        log::info!("Successfully loaded {} releases", entries.len());
                        for entry in &entries {
                            log::info!("Release: {} ({})", entry.version, entry.date);
                        }
                        entries
                    }
                    Err(e) => {
                        log::error!("Failed to load releases: {}", e);
                        default_changelog()
                    }
                }
            });
            let _ = tx.send(result);
        });
        
        self.changelog_loading = false;
    }
    
    /// Экран логина с анимацией
    fn draw_login(&mut self, ui: &mut egui::Ui) {
        // Анимация входа
        self.login_animation = (self.login_animation + 0.05).min(1.0);
        
        ui.vertical_centered(|ui| {
            ui.add_space(80.0 * self.login_animation);
            
            let alpha = (self.login_animation * 255.0) as u8;
            
            ui.label(
                egui::RichText::new("Aporia.cc")
                    .size(64.0)
                    .strong()
                    .color(egui::Color32::from_rgba_unmultiplied(120, 180, 200, alpha))
            );
            
            ui.label(
                egui::RichText::new("Aporia - чит клиент, старающийся получиться более открытым и гибким чем остальные")
                    .size(14.0)
                    .color(egui::Color32::from_rgba_unmultiplied(150, 150, 160, alpha))
            );
            
            ui.add_space(60.0 * self.login_animation);
            
            ui.label(
                egui::RichText::new("Enter your nickname")
                    .size(15.0)
                    .color(egui::Color32::from_rgba_unmultiplied(200, 200, 210, alpha))
            );
            ui.add_space(15.0);
            
            let text_edit = egui::TextEdit::singleline(&mut self.username_input)
                .hint_text("Nickname")
                .desired_width(280.0)
                .font(egui::TextStyle::Heading);
            
            if ui.add(text_edit).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.complete_login();
            }
            
            ui.add_space(25.0);
            
            let button = egui::Button::new(
                egui::RichText::new("Continue")
                    .size(16.0)
                    .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha))
            )
            .min_size(egui::vec2(220.0, 48.0))
            .fill(egui::Color32::from_rgba_unmultiplied(100, 160, 180, (alpha as f32 * 0.8) as u8));
            
            if ui.add(button).clicked() {
                self.complete_login();
            }
        });
    }
    
    fn complete_login(&mut self) {
        if !self.username_input.trim().is_empty() {
            self.config.username = self.username_input.trim().to_string();
            let config_path = self.config.config_path();
            let _ = self.config.save(&config_path);
            self.state = AppState::Main;
            self.main_animation = 0.0;
            self.login_animation = 0.0;
        }
    }
    
    /// Главный экран - новый дизайн
    fn draw_main_content(&mut self, ui: &mut egui::Ui) {
        // Анимация входа главного меню
        self.main_animation = (self.main_animation + 0.08).min(1.0);
        
        ui.horizontal(|ui| {
            // Левая панель - версия и коммиты
            ui.vertical(|ui| {
                ui.add_space(20.0);
                
                // Заголовок
                ui.label(
                    egui::RichText::new("Aporia.cc")
                        .size(32.0)
                        .strong()
                        .color(egui::Color32::from_rgb(120, 180, 200))
                );
                
                ui.label(
                    egui::RichText::new("Aporia - чит клиент, старающийся получиться более открытым и гибким чем остальные")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(150, 150, 160))
                );
                
                ui.add_space(30.0);
                
                // Версия сверху
                ui.label(
                    egui::RichText::new("Version")
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 210))
                );
                ui.add_space(8.0);
                
                let old_version = self.selected_version.clone();
                egui::ComboBox::from_label("")
                    .selected_text(self.selected_version.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_version, McVersion::Fabric, "Fabric 1.21.11 (Modded)");
                        ui.selectable_value(&mut self.selected_version, McVersion::MCP, "MCP v (last build)");
                    });
                
                // Если версия изменилась, загружаем новые коммиты
                if old_version != self.selected_version {
                    self.load_commits();
                }
                
                ui.add_space(30.0);
                
                // Коммиты текущей версии
                ui.label(
                    egui::RichText::new(format!("Latest commits ({})", self.selected_version.branch()))
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 210))
                );
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (i, commit) in self.version_commits.iter().take(10).enumerate() {
                        ui.label(
                            egui::RichText::new(format!("{}. {}", i + 1, commit))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(180, 180, 190))
                        );
                    }
                    
                    if self.version_commits.is_empty() {
                        ui.label(
                            egui::RichText::new("Loading commits...")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 130))
                        );
                    }
                });
                
                ui.add_space(40.0);
                
                // Кнопка запуска внизу - контрастная
                let button_text = if self.is_launching {
                    &self.launch_message
                } else if self.launch_complete {
                    "✓ Launched"
                } else {
                    "▶ Launch"
                };
                
                let button_color = if self.is_launching {
                    egui::Color32::from_rgb(80, 80, 100)
                } else if self.launch_complete {
                    egui::Color32::from_rgb(100, 140, 120)
                } else {
                    egui::Color32::from_rgb(140, 100, 180)
                };
                
                let button = egui::Button::new(
                    egui::RichText::new(button_text)
                        .size(18.0)
                        .color(egui::Color32::from_rgb(255, 255, 255))
                )
                .min_size(egui::vec2(280.0, 70.0))
                .fill(button_color);
                
                let response = ui.add(button);
                
                if response.clicked() && !self.is_launching {
                    self.start_launch();
                }
                
                if self.is_launching {
                    ui.add_space(12.0);
                    ui.add(egui::ProgressBar::new(self.launch_progress).show_percentage());
                }
            });
            
            ui.add_space(40.0);
            
            // Правая панель - релизы Aporia чита
            ui.vertical(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("Aporia Releases")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 210))
                );
                ui.add_space(12.0);
                
                egui::ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
                    for (idx, entry) in self.changelog.iter().enumerate() {
                        let is_selected = idx == self.current_changelog_index;
                        let bg_color = if is_selected {
                            egui::Color32::from_rgba_unmultiplied(50, 50, 70, 150)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(30, 30, 45, 100)
                        };
                        
                        ui.painter().rect_filled(
                            ui.available_rect_before_wrap(),
                            5.0,
                            bg_color,
                        );
                        
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(&entry.version)
                                    .strong()
                                    .color(egui::Color32::from_rgb(120, 180, 200))
                            );
                            ui.label(
                                egui::RichText::new(&entry.date)
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(120, 120, 130))
                            );
                        });
                        
                        ui.add_space(8.0);
                        ui.separator();
                    }
                });
            });
        });
    }
    
    /// Экран настроек
    fn draw_settings_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(25.0);
            ui.heading(
                egui::RichText::new("⚙ Settings")
                    .size(24.0)
                    .color(egui::Color32::from_rgb(200, 200, 210))
            );
            ui.separator();
            ui.add_space(25.0);
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("RAM (MB):").size(14.0));
                ui.add(egui::DragValue::new(&mut self.temp_ram).range(1024..=32768));
            });
            
            ui.add_space(15.0);
            
            ui.checkbox(&mut self.temp_dev_mode, egui::RichText::new("Dev mode (-noverify)").size(14.0));
            
            ui.add_space(40.0);
            
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("Save").size(14.0)).clicked() {
                    self.config.ram_mb = self.temp_ram;
                    self.config.dev_mode = self.temp_dev_mode;
                    let config_path = self.config.config_path();
                    let _ = self.config.save(&config_path);
                    self.state = AppState::Main;
                }
                
                if ui.button(egui::RichText::new("Cancel").size(14.0)).clicked() {
                    self.state = AppState::Main;
                }
            });
        });
    }
    
    /// Начать запуск
    fn start_launch(&mut self) {
        self.is_launching = true;
        self.launch_complete = false;
        self.launch_progress = 0.0;
        self.launch_message = "Preparing...".to_string();
        
        let config = self.config.clone();
        let version = self.selected_version.clone();
        let mods = self.mods.clone();
        
        let (tx, rx) = mpsc::channel::<String>();
        self.rx = Some(rx);
        
        std::thread::spawn(move || {
            let _ = tx.send("Checking Java...".to_string());
            
            match version {
                McVersion::Fabric => {
                    launch_fabric(&config, &mods, &tx);
                }
                McVersion::MCP => {
                    launch_cheat(&config, &tx);
                }
            }
            
            let _ = tx.send("__COMPLETE__".to_string());
        });
    }
}

/// Запуск Fabric версии
fn launch_fabric(config: &Config, mods: &[ModInfo], tx: &mpsc::Sender<String>) {
    let install_path = &config.install_path;
    
    // Проверяем Java
    let java_path = ensure_java(install_path, tx);
    
    let _ = tx.send("Загрузка Fabric...".to_string());
    
    // Загружаем Fabric jar и json
    let versions_path = PathBuf::from(install_path)
        .join("versions")
        .join("Fabric 1.21.11");
    
    let _ = fs::create_dir_all(&versions_path);
    
    let jar_path = versions_path.join("Fabric 1.21.11.jar");
    let json_path = versions_path.join("Fabric 1.21.11.json");
    
    if !jar_path.exists() {
        let url = "https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Fabric%201.21.11/Fabric%201.21.11.jar";
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Downloader::download(url, jar_path.to_str().unwrap()));
    }
    
    if !json_path.exists() {
        let url = "https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Fabric%201.21.11/Fabric%201.21.11.json";
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Downloader::download(url, json_path.to_str().unwrap()));
    }
    
    let _ = tx.send("Загрузка библиотек...".to_string());
    let _ = load_libraries(config, &json_path, tx);
    
    let _ = tx.send("Загрузка модов...".to_string());
    
    let mods_path = PathBuf::from(install_path).join("game").join("mods");
    let _ = fs::create_dir_all(&mods_path);
    
    // Fabric API
    let fabric_api_path = mods_path.join("fabric-api.jar");
    if !fabric_api_path.exists() {
        let url = "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/0.141.2%2B1.21.11/fabric-api-0.141.2%2B1.21.11.jar";
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Downloader::download(url, fabric_api_path.to_str().unwrap()));
    }
    
    // Моды
    let selected_mods: Vec<_> = mods.iter().filter(|m| m.selected).cloned().collect();
    if !selected_mods.is_empty() {
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Downloader::download_mods(mods_path.to_str().unwrap(), &selected_mods));
    }
    
    let _ = tx.send("Распаковка natives...".to_string());
    let _ = extract_natives(config);
    
    let _ = tx.send("Запуск...".to_string());
    let _ = launch_minecraft_fabric(config, &java_path);
}

/// Запуск Cheat версии
fn launch_cheat(config: &Config, tx: &mpsc::Sender<String>) {
    let install_path = &config.install_path;
    
    let _ = tx.send("Проверка Java...".to_string());
    let java_path = ensure_java(install_path, tx);
    
    let _ = tx.send("Загрузка Cheat клиента...".to_string());
    
    // Загружаем Cheat клиент
    let versions_path = PathBuf::from(install_path)
        .join("versions")
        .join("Aporia.client");
    
    let _ = fs::create_dir_all(&versions_path);
    
    let jar_path = versions_path.join("Aporia.client.jar");
    
    if !jar_path.exists() {
        // URL для cheat версии - нужно указать актуальный
        let url = "https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Aporia.client/Aporia.client.jar";
        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Downloader::download(url, jar_path.to_str().unwrap()));
    }
    
    let _ = tx.send("Запуск...".to_string());
    let _ = launch_minecraft_cheat(config, &java_path, &jar_path);
}

/// Обеспечить наличие Java
fn ensure_java(install_path: &str, tx: &mpsc::Sender<String>) -> String {
    let _ = tx.send("Checking Java in PATH...".to_string());
    
    log::info!("Checking for Java in system PATH");
    
    // Пытаемся найти java в PATH
    match Command::new("java")
        .arg("-version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                log::info!("Java found in PATH");
                let _ = tx.send("Java found".to_string());
                return "java".to_string();
            }
        }
        Err(e) => {
            log::warn!("Java not found in PATH: {}", e);
        }
    }
    
    // Если java не найдена в PATH, пытаемся использовать локальную копию
    let java_dir = PathBuf::from(install_path).join("java");
    
    #[cfg(target_os = "windows")]
    let java_exe = java_dir.join("jdk-26").join("bin").join("java.exe");
    
    #[cfg(not(target_os = "windows"))]
    let java_exe = java_dir.join("jdk-26").join("bin").join("java");
    
    if java_exe.exists() {
        log::info!("Using local Java from: {}", java_exe.display());
        let _ = tx.send("Using local Java".to_string());
        return java_exe.to_string_lossy().to_string();
    }
    
    log::error!("Java not found in PATH or local installation");
    let _ = tx.send("Java not found - please install Java".to_string());
    "java".to_string()
}

/// Загрузить библиотеки из JSON
fn load_libraries(config: &Config, json_path: &PathBuf, tx: &mpsc::Sender<String>) -> anyhow::Result<()> {
    let content = fs::read_to_string(json_path)?;
    let json: JsonValue = serde_json::from_str(&content)?;
    
    let libs_path = PathBuf::from(&config.install_path).join("libraries");
    let os_name = utils::get_os_name();
    
    if let Some(libraries) = json.get("libraries").and_then(|v| v.as_array()) {
        for lib in libraries {
            if let Some(name) = lib.get("name").and_then(|v| v.as_str()) {
                if name.contains("ru.legacylauncher") {
                    continue;
                }
                
                if let Some(rules) = lib.get("rules").and_then(|v| v.as_array()) {
                    let mut allowed = false;
                    for rule in rules {
                        if let Some(action) = rule.get("action").and_then(|v| v.as_str()) {
                            if action == "allow" {
                                if let Some(os_rule) = rule.get("os") {
                                    if let Some(rule_os) = os_rule.get("name").and_then(|v| v.as_str()) {
                                        if rule_os == os_name {
                                            allowed = true;
                                        }
                                    }
                                } else {
                                    allowed = true;
                                }
                            }
                        }
                    }
                    if !allowed {
                        continue;
                    }
                }
                
                let base_url = lib.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://libraries.minecraft.net/");
                
                let parts: Vec<&str> = name.split(':').collect();
                if parts.len() < 3 {
                    continue;
                }
                
                let group = parts[0];
                let artifact = parts[1];
                let version = parts[2];
                let classifier = if parts.len() > 3 { parts[3] } else { "" };
                
                let group_path = group.replace('.', "/");
                let mut filename = format!("{}-{}", artifact, version);
                if !classifier.is_empty() {
                    filename.push_str(&format!("-{}", classifier));
                }
                filename.push_str(".jar");
                
                let url = format!("{}/{}/{}/{}/{}", base_url, group_path, artifact, version, filename);
                let local_path = libs_path.join(&group_path).join(artifact).join(version).join(&filename);
                
                if !local_path.exists() {
                    if let Some(parent) = local_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    
                    let _ = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(Downloader::download(&url, local_path.to_str().unwrap()));
                }
            }
        }
    }
    
    // Загружаем assets index
    if let Some(asset_index) = json.get("assetIndex") {
        if let (Some(url), Some(id)) = (
            asset_index.get("url").and_then(|v| v.as_str()),
            asset_index.get("id").and_then(|v| v.as_str())
        ) {
            let index_path = PathBuf::from(&config.install_path)
                .join("assets")
                .join("indexes")
                .join(format!("{}.json", id));
            
            if !index_path.exists() {
                let _ = fs::create_dir_all(index_path.parent().unwrap());
                let _ = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(Downloader::download(url, index_path.to_str().unwrap()));
            }
            
            if let Ok(index_content) = fs::read_to_string(&index_path) {
                if let Ok(index_json) = serde_json::from_str::<JsonValue>(&index_content) {
                    if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
                        let total = objects.len();
                        let _ = tx.send(format!("Ассеты: {}", total));
                        
                        for (i, (_, value)) in objects.iter().enumerate() {
                            if let Some(hash) = value.get("hash").and_then(|v| v.as_str()) {
                                let subdir = &hash[0..2];
                                let object_path = PathBuf::from(&config.install_path)
                                    .join("assets")
                                    .join("objects")
                                    .join(subdir)
                                    .join(hash);
                                
                                if !object_path.exists() {
                                    if let Some(parent) = object_path.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }
                                    
                                    let url = format!("https://resources.download.minecraft.net/{}/{}", subdir, hash);
                                    let _ = tokio::runtime::Runtime::new()
                                        .unwrap()
                                        .block_on(Downloader::download(&url, object_path.to_str().unwrap()));
                                }
                            }
                            
                            if i % 50 == 0 {
                                let _ = tx.send(format!("Ассеты: {}/{}", i, total));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Распаковать natives
fn extract_natives(config: &Config) -> anyhow::Result<()> {
    let libs_path = PathBuf::from(&config.install_path).join("libraries");
    let natives_dir = PathBuf::from(&config.install_path)
        .join("versions")
        .join("Fabric 1.21.11")
        .join("natives");
    
    let _ = fs::create_dir_all(&natives_dir);
    
    let native_pattern = match utils::get_os_name() {
        "windows" => "natives-windows",
        "osx" => "natives-macos",
        _ => "natives-linux",
    };
    
    if libs_path.exists() {
        for entry in walkdir::WalkDir::new(&libs_path) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if entry.path().extension().map(|e| e == "jar").unwrap_or(false) {
                        if entry.path().file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.contains(native_pattern))
                            .unwrap_or(false)
                        {
                            let _ = extract_zip(entry.path(), &natives_dir);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Распаковать ZIP
fn extract_zip(zip_path: &std::path::Path, dest_dir: &std::path::Path) -> anyhow::Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.mangled_name());
        
        if file.name().ends_with('/') {
            let _ = fs::create_dir_all(&outpath);
        } else {
            if let Some(parent) = outpath.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    
    Ok(())
}

/// Запуск Minecraft Fabric
fn launch_minecraft_fabric(config: &Config, java_path: &str) -> anyhow::Result<()> {
    let game_dir = PathBuf::from(&config.install_path).join("game");
    let libs_path = PathBuf::from(&config.install_path).join("libraries");
    let assets_path = PathBuf::from(&config.install_path).join("assets");
    let natives_dir = PathBuf::from(&config.install_path)
        .join("versions")
        .join("Fabric 1.21.11")
        .join("natives");
    
    let _ = fs::create_dir_all(&game_dir);
    
    let mut classpath = vec![
        PathBuf::from(&config.install_path)
            .join("versions")
            .join("Fabric 1.21.11")
            .join("Fabric 1.21.11.jar")
    ];
    
    if libs_path.exists() {
        for entry in walkdir::WalkDir::new(&libs_path) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if entry.path().extension().map(|e| e == "jar").unwrap_or(false) {
                        classpath.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    
    let classpath_str = classpath.iter()
        .filter_map(|p| p.to_str())
        .collect::<Vec<_>>()
        .join(if cfg!(windows) { ";" } else { ":" });
    
    let mut cmd = Command::new(java_path);
    cmd.arg(format!("-Xmx{}M", config.ram_mb));
    cmd.arg(format!("-Djava.library.path={}", natives_dir.display()));
    
    if config.dev_mode {
        cmd.arg("-noverify");
    }
    
    cmd.arg("net.fabricmc.loader.impl.launch.knot.KnotClient");
    cmd.arg("--gameDir").arg(&game_dir);
    cmd.arg("--version").arg("Fabric 1.21.11");
    cmd.arg("--assetsDir").arg(&assets_path);
    cmd.arg("--assetIndex").arg("29");
    cmd.arg("--username").arg(&config.username);
    
    cmd.env("CLASSPATH", &classpath_str);
    cmd.spawn()?;
    
    Ok(())
}

/// Запуск Minecraft Cheat (standalone)
fn launch_minecraft_cheat(config: &Config, java_path: &str, jar_path: &PathBuf) -> anyhow::Result<()> {
    let game_dir = PathBuf::from(&config.install_path).join("game");
    let assets_path = PathBuf::from(&config.install_path).join("assets");
    
    let _ = fs::create_dir_all(&game_dir);
    
    let classpath_str = jar_path.to_str().unwrap();
    
    let mut cmd = Command::new(java_path);
    cmd.arg(format!("-Xmx{}M", config.ram_mb));
    
    if config.dev_mode {
        cmd.arg("-noverify");
    }
    
    // Запуск как standalone без assetIndex
    cmd.arg("-cp").arg(classpath_str);
    cmd.arg("net.minecraft.client.main.Main");
    cmd.arg("--version").arg("mcp");
    cmd.arg("--accessToken").arg("0");
    cmd.arg("--assetsDir").arg(&assets_path);
    cmd.arg("--assetIndex").arg("29");
    cmd.arg("--userProperties").arg("{}");
    cmd.arg("--username").arg(&config.username);
    
    cmd.env("CLASSPATH", classpath_str);
    cmd.spawn()?;
    
    Ok(())
}

/// Парсинг changelog
fn parse_changelog(text: &str) -> Vec<ChangelogEntry> {
    let mut entries = Vec::new();
    let mut current_entry: Option<ChangelogEntry> = None;
    
    for line in text.lines() {
        let line = line.trim();
        
        if line.starts_with("## ") {
            if let Some(entry) = current_entry.take() {
                entries.push(entry);
            }
            
            let parts: Vec<&str> = line[3..].split_whitespace().collect();
            current_entry = Some(ChangelogEntry {
                version: parts[0].to_string(),
                date: parts.get(1).unwrap_or(&"").to_string(),
                changes: Vec::new(),
            });
        } else if line.starts_with("- ") && current_entry.is_some() {
            if let Some(ref mut entry) = current_entry {
                entry.changes.push(line[2..].to_string());
            }
        }
    }
    
    if let Some(entry) = current_entry {
        entries.push(entry);
    }
    
    entries
}

/// Дефолтный changelog
fn default_changelog() -> Vec<ChangelogEntry> {
    vec![
        ChangelogEntry {
            version: "0.3.0".to_string(),
            date: "2026-03-29".to_string(),
            changes: vec![
                "Complete UI redesign".to_string(),
                "Added Cheat version".to_string(),
                "Performance improvements".to_string(),
            ],
        },
        ChangelogEntry {
            version: "0.2.0".to_string(),
            date: "2026-03-28".to_string(),
            changes: vec![
                "Rewritten in Rust".to_string(),
                "Cross-platform GUI".to_string(),
            ],
        },
    ]
}

/// Загрузить релизы Aporia чита
async fn fetch_aporia_releases() -> anyhow::Result<Vec<ChangelogEntry>> {
    log::info!("Fetching Aporia releases from GitHub API");
    
    let client = reqwest::Client::new();
    
    // Получаем релизы Aporia чита
    let releases_url = "https://api.github.com/repos/dakychan/Aporia/releases";
    log::info!("Requesting: {}", releases_url);
    
    let response = client
        .get(releases_url)
        .header("User-Agent", "Aporia-Loader")
        .send()
        .await?;
    
    log::info!("Response status: {}", response.status());
    
    let releases: Vec<JsonValue> = response.json().await?;
    log::info!("Received {} releases", releases.len());
    
    let mut entries = Vec::new();
    
    for (i, release) in releases.iter().take(15).enumerate() {
        if let (Some(tag), Some(body)) = (
            release.get("tag_name").and_then(|v| v.as_str()),
            release.get("body").and_then(|v| v.as_str()),
        ) {
            let published_at = release
                .get("published_at")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .split('T')
                .next()
                .unwrap_or("Unknown");
            
            log::info!("Release {}: {} ({})", i, tag, published_at);
            
            // Парсим чейнджлог из body
            let changes: Vec<String> = body
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+')
                })
                .map(|line| {
                    line.trim()
                        .trim_start_matches('-')
                        .trim_start_matches('*')
                        .trim_start_matches('+')
                        .trim()
                        .to_string()
                })
                .filter(|s| !s.is_empty())
                .collect();
            
            log::info!("  Changes: {}", changes.len());
            
            entries.push(ChangelogEntry {
                version: tag.to_string(),
                date: published_at.to_string(),
                changes: if changes.is_empty() {
                    vec!["Release".to_string()]
                } else {
                    changes
                },
            });
        }
    }
    
    log::info!("Total entries parsed: {}", entries.len());
    
    Ok(if entries.is_empty() {
        log::warn!("No entries found, using default changelog");
        default_changelog()
    } else {
        entries
    })
}

/// Загрузить коммиты для версии из ветки
async fn fetch_commits_for_version(branch: &str) -> anyhow::Result<Vec<String>> {
    let client = reqwest::Client::new();
    
    let commits_url = format!(
        "https://api.github.com/repos/dakychan/Aporia/commits?sha={}&per_page=30",
        branch
    );
    
    let response = client
        .get(&commits_url)
        .header("User-Agent", "Aporia-Loader")
        .send()
        .await?;
    
    let commits: Vec<JsonValue> = response.json().await?;
    let mut messages = Vec::new();
    
    for commit in commits {
        if let Some(msg) = commit
            .get("commit")
            .and_then(|c| c.get("message"))
            .and_then(|m| m.as_str())
        {
            let first_line = msg.lines().next().unwrap_or("").to_string();
            if !first_line.is_empty() {
                messages.push(first_line);
            }
        }
    }
    
    Ok(messages)
}

impl eframe::App for AporiaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Логируем размер окна один раз
        static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOGGED.swap(true, std::sync::atomic::Ordering::SeqCst) {
            let available_rect = ctx.available_rect();
            log::info!("Actual window size: {}x{}", available_rect.width(), available_rect.height());
        }
        
        // Проверяем получение changelog из канала
        if let Some(rx) = &self.changelog_rx {
            if let Ok(entries) = rx.try_recv() {
                log::info!("Received {} changelog entries from thread", entries.len());
                self.changelog = entries;
                self.changelog_rx = None;
            }
        }
        
        // Проверяем получение коммитов из канала
        if let Some(rx) = &self.commits_rx {
            if let Ok(commits) = rx.try_recv() {
                log::info!("Received {} commits from thread", commits.len());
                self.version_commits = commits;
                self.commits_rx = None;
            }
        }
        
        if self.is_launching {
            ctx.request_repaint();
            
            if let Some(rx) = &self.rx {
                while let Ok(msg) = rx.try_recv() {
                    if msg == "__COMPLETE__" {
                        self.launch_complete = true;
                        self.is_launching = false;
                    } else {
                        self.launch_message = msg.clone();
                        
                        if msg.contains("Java") {
                            self.launch_progress = 0.1;
                        } else if msg.contains("Fabric") || msg.contains("Cheat") {
                            self.launch_progress = 0.3;
                        } else if msg.contains("библиотек") || msg.contains("libraries") {
                            self.launch_progress = 0.5;
                        } else if msg.contains("модов") || msg.contains("mods") {
                            self.launch_progress = 0.7;
                        } else if msg.contains("natives") || msg.contains("Запуск") || msg.contains("Launch") {
                            self.launch_progress = 0.9;
                        }
                    }
                }
            }
        }
        
        // Основной контент
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Login => {
                    self.draw_login(ui);
                }
                AppState::Main => {
                    self.draw_main_content(ui);
                }
                AppState::Settings => {
                    self.draw_settings_content(ui);
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    
    log::info!("Starting Aporia Loader");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1200.0, 700.0]),
        ..Default::default()
    };
    
    log::info!("Window size: 1600x900");
    
    eframe::run_native(
        "Aporia Loader",
        options,
        Box::new(|cc| {
            log::info!("Creating app instance");
            Ok(Box::new(AporiaApp::new(cc)))
        }),
    )
}
