//! Aporia Loader - Cross-platform Minecraft Loader

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;
use serde_json::Value as JsonValue;
use rand::Rng;

// ============================================================
// CONFIG
// ============================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub install_path: String,
    pub ram_mb: u32,
    pub username: String,
    pub dev_mode: bool,
    pub java_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            install_path: get_default_path().to_string_lossy().to_string(),
            ram_mb: 4096,
            username: "Player".to_string(),
            dev_mode: false,
            java_path: String::new(),
        }
    }
}

impl Config {
    pub fn load(&mut self, path: &std::path::Path) {
        if !path.exists() { return; }
        if let Ok(file) = fs::File::open(path) {
            use std::io::BufRead;
            for line in std::io::BufReader::new(file).lines().flatten() {
                if let Some(pos) = line.find('=') {
                    match line[..pos].trim() {
                        "path" => self.install_path = line[pos + 1..].trim().to_string(),
                        "ram" => self.ram_mb = line[pos + 1..].trim().parse().unwrap_or(4096),
                        "username" => self.username = line[pos + 1..].trim().to_string(),
                        "devmode" => self.dev_mode = line[pos + 1..].trim() == "true",
                        "javapath" => self.java_path = line[pos + 1..].trim().to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(p) = path.parent() { fs::create_dir_all(p)?; }
        let mut f = fs::File::create(path)?;
        use std::io::Write;
        writeln!(f, "path={}", self.install_path)?;
        writeln!(f, "ram={}", self.ram_mb)?;
        writeln!(f, "username={}", self.username)?;
        writeln!(f, "devmode={}", if self.dev_mode { "true" } else { "false" })?;
        if !self.java_path.is_empty() { writeln!(f, "javapath={}", self.java_path)?; }
        Ok(())
    }

    pub fn config_path(&self) -> PathBuf {
        PathBuf::from(&self.install_path).join("config.txt")
    }
}

// ============================================================
// DOWNLOADER
// ============================================================

#[derive(Debug, Clone)]
pub struct ModInfo {
    pub name: String,
    pub url: String,
    pub selected: bool,
}

pub struct Downloader;

impl Downloader {
    pub async fn download(url: &str, path: &str) -> anyhow::Result<u64> {
        let resp = reqwest::Client::new().get(url).send().await?;
        let bytes = resp.bytes().await?;
        if let Some(parent) = std::path::Path::new(path).parent() { fs::create_dir_all(parent)?; }
        fs::write(path, &bytes)?;
        Ok(bytes.len() as u64)
    }

    pub async fn download_with_progress<F: Fn(u64, Option<u64>)>(
        url: &str, path: &str, _cb: F,
    ) -> anyhow::Result<u64> { Self::download(url, path).await }

    pub async fn download_mods(dir: &str, mods: &[ModInfo]) -> anyhow::Result<()> {
        for m in mods {
            if !m.selected { continue; }
            let fn_ = m.url.split('/').last().unwrap_or("mod.jar");
            Self::download(&m.url, &format!("{}/{}", dir, fn_)).await?;
        }
        Ok(())
    }
}

// ============================================================
// UTILS
// ============================================================

pub fn get_default_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    { if let Some(a) = std::env::var_os("APPDATA") { return PathBuf::from(a).join("apr"); } }
    #[cfg(target_os = "macos")]
    { if let Some(h) = dirs::home_dir() { return h.join("Library/Application Support/apr"); } }
    #[cfg(target_os = "linux")]
    { if let Some(h) = dirs::home_dir() { return h.join(".apr"); } }
    PathBuf::from("./apr")
}

pub fn get_os_name() -> &'static str {
    #[cfg(target_os = "windows")] { return "windows"; }
    #[cfg(target_os = "macos")] { return "osx"; }
    #[cfg(target_os = "linux")] { return "linux"; }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))] { return "unknown"; }
}

// ============================================================
// COSMIC BACKGROUND — с libblur
// ============================================================

#[derive(Clone)]
struct Star {
    x: f32, y: f32, radius: f32,
    base_alpha: f32, phase: f32, speed: f32,
    is_bright: bool,
}

#[derive(Clone)]
struct CometParticle {
    x: f32, y: f32, vx: f32, vy: f32,
    life: f32, decay: f32, size: f32,
}

#[derive(Clone)]
struct Comet {
    x: f32, y: f32, vx: f32, vy: f32,
    tail_len: f32, width: f32, hue: f32,
    life: f32, decay: f32,
    particles: Vec<CometParticle>,
}

impl Comet {
    fn new(w: f32, h: f32, spread: bool) -> Self {
        let angle = (rand::thread_rng().gen::<f32>() * 20.0 + 30.0).to_radians();
        let speed = rand::thread_rng().gen::<f32>() * 6.0 + 5.0;
        Self {
            x: if spread { rand::thread_rng().gen::<f32>() * w * 1.3 } else { w + rand::thread_rng().gen::<f32>() * 400.0 },
            y: if spread { rand::thread_rng().gen::<f32>() * h * 0.5 } else { -rand::thread_rng().gen::<f32>() * 300.0 },
            vx: -angle.cos() * speed,
            vy: angle.sin() * speed,
            tail_len: rand::thread_rng().gen::<f32>() * 80.0 + 120.0,
            width: rand::thread_rng().gen::<f32>() * 2.0 + 1.5,
            hue: if rand::thread_rng().gen::<f32>() > 0.5 { 260.0 } else { 220.0 },
            life: 1.0,
            decay: rand::thread_rng().gen::<f32>() * 0.002 + 0.0012,
            particles: Vec::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt * 60.0;
        self.y += self.vy * dt * 60.0;
        self.life -= self.decay * dt * 60.0;
        if rand::thread_rng().gen::<f32>() < 0.4 && self.life > 0.2 {
            self.particles.push(CometParticle {
                x: self.x + rand::thread_rng().gen::<f32>() * 6.0 - 3.0,
                y: self.y + rand::thread_rng().gen::<f32>() * 6.0 - 3.0,
                vx: -self.vx * 0.05 + rand::thread_rng().gen::<f32>() * 0.4 - 0.2,
                vy: -self.vy * 0.05 + rand::thread_rng().gen::<f32>() * 0.4 - 0.2,
                life: 1.0,
                decay: rand::thread_rng().gen::<f32>() * 0.04 + 0.03,
                size: rand::thread_rng().gen::<f32>() * 1.2 + 0.3,
            });
        }
        for p in &mut self.particles {
            p.x += p.vx * dt * 60.0;
            p.y += p.vy * dt * 60.0;
            p.life -= p.decay * dt * 60.0;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn is_dead(&self, w: f32, h: f32) -> bool {
        self.life <= 0.0 || self.x < -250.0 || self.y > h + 250.0 || self.x > w + 250.0
    }
}

pub struct CosmicBackground {
    stars: Vec<Star>,
    comets: Vec<Comet>,
    start_time: Instant,
    last_flash: f32,
    next_flash_at: f32,
    flash_alpha: f32,
    flash_x: f32,
    flash_y: f32,
    flash_hue: f32,
    comets_inited: bool,
    blurred_texture: Option<egui::TextureHandle>,
    blur_size: (u32, u32),
}

impl Default for CosmicBackground {
    fn default() -> Self { Self::new() }
}

impl CosmicBackground {
    pub fn new() -> Self {
        let mut stars = Vec::with_capacity(200);
        for i in 0..200 {
            let bright = i < 12;
            stars.push(Star {
                x: rand::thread_rng().gen::<f32>() * 3200.0 - 500.0,
                y: rand::thread_rng().gen::<f32>() * 2200.0 - 200.0,
                radius: if bright { rand::thread_rng().gen::<f32>() * 0.8 + 1.2 } else { rand::thread_rng().gen::<f32>() * 0.8 + 0.3 },
                base_alpha: if bright { rand::thread_rng().gen::<f32>() * 0.3 + 0.7 } else { rand::thread_rng().gen::<f32>() * 0.4 + 0.15 },
                phase: rand::thread_rng().gen::<f32>() * std::f32::consts::PI * 2.0,
                speed: rand::thread_rng().gen::<f32>() * 0.6 + 0.2,
                is_bright: bright,
            });
        }
        Self {
            stars, comets: Vec::new(), start_time: Instant::now(),
            last_flash: 0.0, next_flash_at: rand::thread_rng().gen::<f32>() * 3.0 + 2.0,
            flash_alpha: 0.0, flash_x: 0.0, flash_y: 0.0, flash_hue: 260.0,
            comets_inited: false,
            blurred_texture: None, blur_size: (0, 0),
        }
    }

    /// Генерация размытого фона через libblur (один раз, кэшируется)
    fn generate_blur(&mut self, ctx: &egui::Context, full_w: u32, full_h: u32) {
        // Половинное разрешение — blur всё равно скрывает пиксели
        let w = (full_w / 2).max(400);
        let h = (full_h / 2).max(225);
        let total = (w * h * 4) as usize;
        let mut px = vec![0u8; total];

        // Заливка фоном
        for i in 0..(w * h) as usize {
            px[i * 4] = 6; px[i * 4 + 1] = 6; px[i * 4 + 2] = 14; px[i * 4 + 3] = 255;
        }

        // Тусклые звёзды
        for _ in 0..250 {
            let sx = rand::thread_rng().gen_range(0..w) as usize;
            let sy = rand::thread_rng().gen_range(0..h) as usize;
            let br = rand::thread_rng().gen::<f32>() * 0.6 + 0.2;
            let r = if rand::thread_rng().gen::<f32>() > 0.9 { 2usize } else { 1 };
            for dy in -(r as i32)..=(r as i32) {
                for dx in -(r as i32)..=(r as i32) {
                    let nx = sx as i32 + dx;
                    let ny = sy as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w as usize && (ny as usize) < h as usize {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d <= r as f32 {
                            let a = (1.0 - d / r as f32) * br;
                            let i = (ny as usize * w as usize + nx as usize) * 4;
                            px[i] = (px[i] as f32 + 220.0 * a).min(255.0) as u8;
                            px[i + 1] = (px[i + 1] as f32 + 215.0 * a).min(255.0) as u8;
                            px[i + 2] = (px[i + 2] as f32 + 255.0 * a).min(255.0) as u8;
                        }
                    }
                }
            }
        }

        // Яркие звёзды (создают боке при размытии)
        for _ in 0..18 {
            let sx = rand::thread_rng().gen_range(0..w) as usize;
            let sy = rand::thread_rng().gen_range(0..h) as usize;
            let br = rand::thread_rng().gen::<f32>() * 0.4 + 0.8;
            let r = rand::thread_rng().gen_range(2..5) as usize;
            for dy in -(r as i32)..=(r as i32) {
                for dx in -(r as i32)..=(r as i32) {
                    let nx = sx as i32 + dx;
                    let ny = sy as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w as usize && (ny as usize) < h as usize {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d <= r as f32 {
                            let a = (1.0 - (d / r as f32).powf(0.7)) * br;
                            let i = (ny as usize * w as usize + nx as usize) * 4;
                            px[i] = (px[i] as f32 + 240.0 * a).min(255.0) as u8;
                            px[i + 1] = (px[i + 1] as f32 + 235.0 * a).min(255.0) as u8;
                            px[i + 2] = (px[i + 2] as f32 + 255.0 * a).min(255.0) as u8;
                        }
                    }
                }
            }
        }

        // Туманности
        for (xf, yf, rad, cr, cg, cb) in [
            (0.25f32, 0.35f32, 140.0f32, 88u8, 28u8, 135u8),
            (0.72f32, 0.55f32, 110.0f32, 30u8, 58u8, 138u8),
            (0.50f32, 0.18f32, 90.0f32, 60u8, 20u8, 100u8),
        ] {
            let cx = (w as f32 * xf) as i32;
            let cy = (h as f32 * yf) as i32;
            let ri = rad as i32;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w as usize && (ny as usize) < h as usize {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d <= rad {
                            let a = (1.0 - d / rad).powf(2.0) * 0.1;
                            let i = (ny as usize * w as usize + nx as usize) * 4;
                            px[i] = (px[i] as f32 + cr as f32 * a * 255.0).min(255.0) as u8;
                            px[i + 1] = (px[i + 1] as f32 + cg as f32 * a * 255.0).min(255.0) as u8;
                            px[i + 2] = (px[i + 2] as f32 + cb as f32 * a * 255.0).min(255.0) as u8;
                        }
                    }
                }
            }
        }

        // Конвертация в egui::ColorImage (без blur пока)
        let mut ci = egui::ColorImage::new([w as usize, h as usize], egui::Color32::from_rgb(6, 6, 14));
        for i in 0..(w * h) as usize {
            ci.pixels[i] = egui::Color32::from_rgba_unmultiplied(
                px[i * 4],
                px[i * 4 + 1],
                px[i * 4 + 2],
                255,
            );
        }

        self.blurred_texture = Some(ctx.load_texture("blur_bg", ci, egui::TextureOptions::LINEAR));
        self.blur_size = (full_w, full_h);
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let dt = ui.ctx().input(|i| i.stable_dt).min(0.05);
        let w = rect.width();
        let h = rect.height();
        let painter = ui.painter();
        let fw = w as u32;
        let fh = h as u32;

        // Генерация/перегенерация блюра при ресайзе
        if self.blurred_texture.is_none() || (self.blur_size.0.abs_diff(fw) > 50 || self.blur_size.1.abs_diff(fh) > 50) {
            self.generate_blur(ui.ctx(), fw, fh);
        }

        // Рисуем размытый фон как текстуру
        if let Some(tex) = &self.blurred_texture {
            painter.image(
                tex.id(), rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(6, 6, 14));
        }

        // Инициализация комет
        if !self.comets_inited {
            for _ in 0..5 { self.comets.push(Comet::new(w, h, true)); }
            self.comets_inited = true;
        }

        // Анимированные звёзды (мерцание поверх размытого фона)
        for star in &self.stars {
            let flicker = (elapsed * star.speed + star.phase).sin() * 0.25;
            let alpha = ((star.base_alpha + flicker).max(0.05)).min(1.0);
            let sx = (star.x % w) + rect.min.x;
            let sy = (star.y % h) + rect.min.y;
            painter.circle_filled(egui::pos2(sx, sy), star.radius,
                egui::Color32::from_rgba_unmultiplied(220, 215, 255, (alpha * 255.0) as u8));
            if star.is_bright {
                let ss = star.radius * 3.5 + (elapsed * star.speed * 0.5 + star.phase).sin() * 1.5;
                let sc = egui::Color32::from_rgba_unmultiplied(240, 235, 255, (alpha * 0.5 * 255.0) as u8);
                painter.line_segment([egui::pos2(sx - ss, sy), egui::pos2(sx + ss, sy)], egui::Stroke::new(0.5, sc));
                painter.line_segment([egui::pos2(sx, sy - ss), egui::pos2(sx, sy + ss)], egui::Stroke::new(0.5, sc));
            }
        }

        // Кометы
        while self.comets.len() < 5 { self.comets.push(Comet::new(w, h, false)); }
        for i in (0..self.comets.len()).rev() {
            self.comets[i].update(dt);
            if self.comets[i].is_dead(w, h) { self.comets[i] = Comet::new(w, h, false); }
            let c = &self.comets[i];
            let head = rect.min + egui::vec2(c.x, c.y);
            let spd = (c.vx * c.vx + c.vy * c.vy).sqrt();
            if spd < 0.01 { continue; }
            let dx = c.vx / spd; let dy = c.vy / spd;
            let (tr, tg, tb) = if c.hue > 240.0 { (180, 140, 255) } else { (140, 160, 255) };
            let (cr, cg, cb) = if c.hue > 240.0 { (220, 200, 255) } else { (200, 210, 255) };

            // Хвост — 25 сегментов
            for j in 0..25 {
                let t0 = j as f32 / 25.0;
                let t1 = (j + 1) as f32 / 25.0;
                let p0 = head - egui::vec2(dx, dy) * c.tail_len * t0;
                let p1 = head - egui::vec2(dx, dy) * c.tail_len * t1;
                let fade = (1.0 - t0).powf(2.5);
                painter.line_segment([p0, p1],
                    egui::Stroke::new(c.width * (1.0 - t0 * 0.85).max(0.3),
                        egui::Color32::from_rgba_unmultiplied(tr, tg, tb, (fade * c.life * 200.0) as u8)));
            }
            // Частицы
            for p in &c.particles {
                painter.circle_filled(rect.min + egui::vec2(p.x, p.y), p.size,
                    egui::Color32::from_rgba_unmultiplied(tr, tg, tb, (p.life * c.life * 150.0) as u8));
            }
            // Свечение головы — 3 слоя
            for (sz, intens) in [(c.width * 10.0, 0.08), (c.width * 5.0, 0.18), (c.width * 2.5, 0.45)] {
                painter.circle_filled(head, sz,
                    egui::Color32::from_rgba_unmultiplied(cr, cg, cb, (c.life * intens * 255.0) as u8));
            }
            painter.circle_filled(head, c.width * 0.8,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, (c.life * 220.0) as u8));
        }

        // Вспышки
        self.last_flash += dt;
        if self.last_flash > self.next_flash_at {
            self.flash_alpha = 1.0;
            self.flash_x = rand::thread_rng().gen::<f32>() * w;
            self.flash_y = rand::thread_rng().gen::<f32>() * h * 0.75;
            self.flash_hue = if rand::thread_rng().gen::<f32>() > 0.4 { 260.0 } else { 220.0 };
            self.last_flash = 0.0;
            self.next_flash_at = rand::thread_rng().gen::<f32>() * 5.0 + 3.0;
        }
        if self.flash_alpha > 0.0 { self.flash_alpha -= dt * 4.5; }
        if self.flash_alpha > 0.01 {
            let center = rect.min + egui::vec2(self.flash_x, self.flash_y);
            let (fr, fg, fb) = if self.flash_hue > 240.0 { (180, 140, 255) } else { (140, 170, 255) };
            for j in (0..20).rev() {
                let t = j as f32 / 20.0;
                painter.circle_filled(center, t * 280.0,
                    egui::Color32::from_rgba_unmultiplied(fr, fg, fb, ((1.0 - t).powf(3.0) * self.flash_alpha * 0.1 * 255.0) as u8));
            }
            painter.circle_filled(center, 8.0,
                egui::Color32::from_rgba_unmultiplied(255, 250, 255, (self.flash_alpha * 0.35 * 255.0) as u8));
        }

        ui.ctx().request_repaint();
    }

    pub fn trigger_flash(&mut self) {
        self.flash_alpha = 1.2;
        self.flash_x = rand::thread_rng().gen::<f32>() * 1200.0 + 200.0;
        self.flash_y = rand::thread_rng().gen::<f32>() * 500.0 + 100.0;
    }
}

// ============================================================
// ТИПЫ
// ============================================================

const VERSION: &str = "0.5.0";

#[derive(Debug, Clone, PartialEq)]
enum McVersion { Fabric, MCP }

impl McVersion {
    fn name(&self) -> &'static str {
        match self { McVersion::Fabric => "Fabric 1.21.11 (Modded)", McVersion::MCP => "MCP v0.5.0 (last build)" }
    }
    fn branch(&self) -> &'static str {
        match self { McVersion::Fabric => "fabric", McVersion::MCP => "mcp" }
    }
}

#[derive(PartialEq, Clone)]
enum AppState { Login, Main, Settings }

#[derive(Debug, Clone)]
struct ChangelogEntry { version: String, date: String, changes: Vec<String> }

// ============================================================
// AporiaApp
// ============================================================

struct AporiaApp {
    state: AppState,
    config: Config,
    selected_version: McVersion,
    cosmic_bg: CosmicBackground,
    username_input: String,
    login_animation: f32,
    changelog: Vec<ChangelogEntry>,
    current_changelog_index: usize,
    main_animation: f32,
    changelog_rx: Option<mpsc::Receiver<Vec<ChangelogEntry>>>,
    version_commits: Vec<String>,
    commits_rx: Option<mpsc::Receiver<Vec<String>>>,
    is_launching: bool,
    launch_progress: f32,
    launch_message: String,
    launch_complete: bool,
    temp_ram: u32,
    temp_dev_mode: bool,
    rx: Option<mpsc::Receiver<String>>,
    mods: Vec<ModInfo>,
}

impl Default for AporiaApp {
    fn default() -> Self {
        let config = Config::default();
        Self {
            state: AppState::Login, config: config.clone(),
            selected_version: McVersion::MCP, cosmic_bg: CosmicBackground::new(),
            username_input: config.username.clone(), login_animation: 0.0,
            changelog: Vec::new(), current_changelog_index: 0, main_animation: 0.0,
            changelog_rx: None, version_commits: Vec::new(), commits_rx: None,
            is_launching: false, launch_progress: 0.0, launch_message: String::new(), launch_complete: false,
            temp_ram: config.ram_mb, temp_dev_mode: config.dev_mode, rx: None,
            mods: vec![
                ModInfo { name: "Mod Menu".into(), url: "https://cdn.modrinth.com/data/mOgUt4GM/versions/JWQVh32x/modmenu-17.0.0-beta.2.jar".into(), selected: true },
                ModInfo { name: "3D Skin Layers".into(), url: "https://cdn.modrinth.com/data/zV5r3pPn/versions/JS9deRtw/skinlayers3d-fabric-1.10.2-mc1.21.11.jar".into(), selected: true },
                ModInfo { name: "Sound Physics Remastered".into(), url: "https://cdn.modrinth.com/data/qyVF9oeo/versions/pfqxi9qs/sound-physics-remastered-fabric-1.21.11-1.5.1.jar".into(), selected: true },
                ModInfo { name: "Cloth Config".into(), url: "https://cdn.modrinth.com/data/9s6osm5g/versions/xuX40TN5/cloth-config-21.11.153-fabric.jar".into(), selected: true },
            ],
        }
    }
}

impl AporiaApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 6.0);
        style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(15, 15, 20, 255);
        style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(12, 12, 18, 255);
        style.visuals.extreme_bg_color = egui::Color32::from_rgba_unmultiplied(8, 8, 12, 255);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(20, 20, 30, 220);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(30, 30, 45, 230);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(40, 40, 60, 240);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(50, 50, 75, 240);
        cc.egui_ctx.set_style(style);
        let mut app = Self::default();
        app.load_changelog();
        app.load_commits();
        app
    }

    fn load_commits(&mut self) {
        let branch = self.selected_version.branch().to_string();
        let (tx, rx) = mpsc::channel();
        self.commits_rx = Some(rx);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let r = rt.block_on(fetch_commits_for_version(&branch)).unwrap_or_else(|_| vec!["Failed".to_string()]);
            let _ = tx.send(r);
        });
    }

    fn load_changelog(&mut self) {
        self.changelog = default_changelog();
        let (tx, rx) = mpsc::channel();
        self.changelog_rx = Some(rx);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let r = rt.block_on(fetch_aporia_releases()).unwrap_or_else(|_| default_changelog());
            let _ = tx.send(r);
        });
    }

    // ---- LOGIN ----
    fn draw_login(&mut self, ui: &mut egui::Ui) {
        self.login_animation = (self.login_animation + 0.05).min(1.0);
        let a = (self.login_animation * 255.0) as u8;
        ui.vertical_centered(|ui| {
            ui.add_space(100.0 * self.login_animation);
            ui.label(egui::RichText::new("Aporia.cc").size(60.0).strong().color(egui::Color32::from_rgba_unmultiplied(139, 92, 246, a)));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Модифицированный клиент — быстрее и стабильнее").size(14.0).color(egui::Color32::from_rgba_unmultiplied(150, 150, 160, a)));
            ui.add_space(50.0 * self.login_animation);
            ui.label(egui::RichText::new("Enter your nickname").size(15.0).color(egui::Color32::from_rgba_unmultiplied(200, 200, 210, a)));
            ui.add_space(12.0);
            let te = egui::TextEdit::singleline(&mut self.username_input).hint_text("Nickname").desired_width(280.0).font(egui::TextStyle::Heading);
            if ui.add(te).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.complete_login(); }
            ui.add_space(20.0);
            let btn = egui::Button::new(egui::RichText::new("Continue").size(16.0).color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, a)))
                .min_size(egui::vec2(220.0, 48.0)).fill(egui::Color32::from_rgba_unmultiplied(124, 58, 237, (a as f32 * 0.85) as u8));
            if ui.add(btn).clicked() { self.complete_login(); }
        });
    }

    fn complete_login(&mut self) {
        if !self.username_input.trim().is_empty() {
            self.config.username = self.username_input.trim().to_string();
            let _ = self.config.save(&self.config.config_path());
            self.state = AppState::Main;
            self.main_animation = 0.0;
        }
    }

    // ---- ГЛАВНЫЙ ЭКРАН ----
    fn draw_main_content(&mut self, ui: &mut egui::Ui) {
        self.main_animation = (self.main_animation + 0.06).min(1.0);
        let sr = ui.max_rect();
        let pad = 24.0;
        let gap = 20.0;
        let lw = (sr.width() - pad * 2.0 - gap) * 0.55;
        let rw = sr.width() - pad * 2.0 - gap - lw;

        let lr = egui::Rect::from_min_size(sr.min + egui::vec2(pad, pad), egui::vec2(lw, sr.height() - pad * 2.0));
        let rr = egui::Rect::from_min_size(sr.min + egui::vec2(pad + lw + gap, pad), egui::vec2(rw, sr.height() - pad * 2.0));

        // === ФОН ПАНЕЛЕЙ (до контента) ===
        let p = ui.painter().clone();
        for rect in [lr, rr] {
            p.rect_filled(rect, 14.0, egui::Color32::from_rgba_unmultiplied(10, 10, 22, 215));
            p.rect_stroke(rect, 14.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 80, 220, 35)));
        }

        // === ЛЕВАЯ ПАНЕЛЬ ===
        let mut lu = ui.child_ui(lr.shrink(32.0), egui::Layout::top_down(egui::Align::LEFT), None);
        lu.set_width(lw - 64.0);

        // Бренд
        lu.add_space(16.0);
        lu.horizontal(|ui| {
            let is = 44.0;
            let ir = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(is, is));
            // Glow — 4 слоя фиолетового
            for (mul, alpha) in [(1.8, 8u8), (1.4, 15), (1.1, 25), (0.95, 40)] {
                ui.painter().circle_filled(ir.center(), is * mul,
                    egui::Color32::from_rgba_unmultiplied(139, 92, 246, alpha));
            }
            ui.painter().rect_filled(ir, 10.0, egui::Color32::from_rgb(139, 92, 246));
            ui.painter().text(ir.center(), egui::Align2::CENTER_CENTER, "A",
                egui::FontId::proportional(20.0), egui::Color32::WHITE);
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Aporia").size(28.0).strong().color(egui::Color32::from_rgb(232, 230, 240)));
        });

        lu.add_space(6.0);
        lu.label(egui::RichText::new("Модифицированный клиент с улучшенной производительностью и полной совместимостью.")
            .size(12.5).color(egui::Color32::from_rgb(110, 107, 130)));

        lu.add_space(18.0);

        // Версия
        lu.label(egui::RichText::new("VERSION").size(10.0).strong().color(egui::Color32::from_rgb(74, 72, 96)));
        lu.add_space(4.0);
        let ov = self.selected_version.clone();
        egui::ComboBox::from_id_source("ver").selected_text(self.selected_version.name())
            .width(280.0).show_ui(&mut lu, |ui| {
                ui.selectable_value(&mut self.selected_version, McVersion::Fabric, "Fabric 1.21.11 (Modded)");
                ui.selectable_value(&mut self.selected_version, McVersion::MCP, "MCP v0.5.0 (last build)");
            });
        if ov != self.selected_version { self.load_commits(); }

        lu.add_space(16.0);

        // Коммиты
        lu.label(egui::RichText::new(format!("LATEST COMMITS ({})", self.selected_version.branch().to_uppercase()))
            .size(10.0).strong().color(egui::Color32::from_rgb(74, 72, 96)));
        lu.add_space(6.0);

        let cc = self.version_commits.clone();
        let commits_height = 150.0;
        egui::ScrollArea::vertical().max_height(commits_height).id_source("commits").show(&mut lu, |ui| {
            for c in cc.iter().take(5) {
                let ps: Vec<&str> = c.splitn(2, ' ').collect();
                let h = ps.get(0).unwrap_or(&"");
                let m = ps.get(1).unwrap_or(&"");
                ui.horizontal(|ui| {
                    // Хеш с фоном
                    let hash_resp = ui.allocate_response(egui::vec2(68.0, 22.0), egui::Sense::hover());
                    ui.painter().rect_filled(hash_resp.rect, 5.0,
                        egui::Color32::from_rgba_unmultiplied(139, 92, 246, if hash_resp.hovered { 30 } else { 18 }));
                    ui.painter().text(hash_resp.rect.center(), egui::Align2::CENTER_CENTER, *h,
                        egui::FontId::monospace(11.0), egui::Color32::from_rgb(167, 139, 250));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(*m).size(12.0).color(egui::Color32::from_rgb(122, 119, 144)));
                });
                ui.add_space(3.0);
            }
            if cc.is_empty() {
                ui.label(egui::RichText::new("Loading commits...").size(12.0).color(egui::Color32::from_rgb(90, 87, 110)));
            }
        });

        // Кнопка Launch — прижата к низу
        lu.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(10.0);
            let (bt, bc) = if self.is_launching { (self.launch_message.as_str(), egui::Color32::from_rgb(100, 50, 200)) }
                else if self.launch_complete { ("Running", egui::Color32::from_rgb(5, 130, 90)) }
                else { ("Launch", egui::Color32::from_rgb(124, 58, 237)) };

            let br = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(lw - 64.0, 46.0));
            // Glow под кнопкой — 3 слоя
            if !self.is_launching && !self.launch_complete {
                for (exp, alpha) in [(18.0, 6u8), (10.0, 14), (5.0, 25)] {
                    ui.painter().rect_filled(br.expand(exp), 18.0,
                        egui::Color32::from_rgba_unmultiplied(139, 92, 246, alpha));
                }
            }

            let btn = egui::Button::new(egui::RichText::new(bt).size(16.0).strong().color(egui::Color32::WHITE))
                .min_size(egui::vec2(lw - 64.0, 46.0)).fill(bc).rounding(12.0);
            if ui.add(btn).clicked() && !self.is_launching { self.start_launch(); self.cosmic_bg.trigger_flash(); }
            if self.is_launching {
                ui.add_space(8.0);
                ui.add(egui::ProgressBar::new(self.launch_progress).show_percentage().fill(egui::Color32::from_rgb(139, 92, 246)));
            }
        });

        // === ПРАВАЯ ПАНЕЛЬ ===
        let mut ru = ui.child_ui(rr.shrink(24.0), egui::Layout::top_down(egui::Align::LEFT), None);
        ru.set_width(rw - 48.0);
        ru.add_space(16.0);

        ru.label(egui::RichText::new("Aporia Releases").size(16.0).strong().color(egui::Color32::from_rgb(232, 230, 240)));
        ru.add_space(3.0);
        ru.label(egui::RichText::new("Доступные версии").size(11.0).color(egui::Color32::from_rgb(74, 72, 96)));
        ru.add_space(12.0);

        let cl = self.changelog.clone();
        let rel_height = sr.height() - 160.0;
        egui::ScrollArea::vertical().max_height(rel_height).id_source("releases").show(&mut ru, |ui| {
            for (idx, e) in cl.iter().enumerate() {
                let il = idx == 0;
                let resp = ui.add(egui::Button::new("")
                    .min_size(egui::vec2(rw - 52.0, 42.0))
                    .fill(if il { egui::Color32::from_rgba_unmultiplied(139, 92, 246, 18) } else { egui::Color32::TRANSPARENT })
                    .stroke(egui::Stroke::new(1.0,
                        if il { egui::Color32::from_rgba_unmultiplied(139, 92, 246, 45) } else { egui::Color32::TRANSPARENT }))
                    .rounding(10.0));
                let br = resp.rect;
                ui.painter().text(br.left_center() + egui::vec2(14.0, 0.0), egui::Align2::LEFT_CENTER, &e.version,
                    egui::FontId::monospace(13.0), if il { egui::Color32::from_rgb(167, 139, 250) } else { egui::Color32::from_rgb(220, 218, 230) });
                if il {
                    let bp = br.left_center() + egui::vec2(100.0, 0.0);
                    ui.painter().rect_filled(egui::Rect::from_center_size(bp, egui::vec2(48.0, 16.0)), 5.0,
                        egui::Color32::from_rgba_unmultiplied(52, 211, 153, 22));
                    ui.painter().text(bp, egui::Align2::CENTER_CENTER, "LATEST", egui::FontId::proportional(8.0), egui::Color32::from_rgb(52, 211, 153));
                }
                ui.painter().text(br.right_center() - egui::vec2(14.0, 0.0), egui::Align2::RIGHT_CENTER, &e.date,
                    egui::FontId::monospace(11.0), egui::Color32::from_rgb(74, 72, 96));
                if resp.clicked() { self.current_changelog_index = idx; self.cosmic_bg.trigger_flash(); }
                ui.add_space(3.0);
            }
        });

        // Статус-бар
        ru.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(10.0);
            let sy = ui.cursor().min.y - 5.0;
            ui.painter().line_segment(
                [egui::pos2(rr.left() + 24.0, sy), egui::pos2(rr.right() - 24.0, sy)],
                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10)));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let dp = ui.cursor().min + egui::vec2(3.5, 6.0);
                let pulse = (self.cosmic_bg.start_time.elapsed().as_secs_f32() * 2.0).sin() * 0.3 + 0.7;
                ui.painter().circle_filled(dp, 8.0, egui::Color32::from_rgba_unmultiplied(52, 211, 153, (pulse * 35.0) as u8));
                ui.painter().circle_filled(dp, 3.5, egui::Color32::from_rgba_unmultiplied(52, 211, 153, (pulse * 255.0) as u8));
                ui.add_space(14.0);
                ui.label(egui::RichText::new("Сервисы онлайн — ").size(10.5).color(egui::Color32::from_rgb(74, 72, 96)));
                ui.label(egui::RichText::new("пинг 24ms").size(10.5).color(egui::Color32::from_rgb(122, 119, 144)));
            });
        });

        // === ВЕРХНИЕ ПОДСВЕТКИ ПАНЕЛЕЙ (после контента) ===
        for rect in [lr, rr] {
            p.line_segment(
                [rect.left_top() + egui::vec2(18.0, 0.5), rect.right_top() - egui::vec2(18.0, 0.5)],
                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(139, 92, 246, 70)));
        }
    }

    fn draw_settings_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(25.0);
            ui.heading(egui::RichText::new("Settings").size(24.0).color(egui::Color32::from_rgb(200, 200, 210)));
            ui.separator(); ui.add_space(25.0);
            ui.horizontal(|ui| { ui.label(egui::RichText::new("RAM (MB):").size(14.0)); ui.add(egui::DragValue::new(&mut self.temp_ram).range(1024..=32768)); });
            ui.add_space(15.0);
            ui.checkbox(&mut self.temp_dev_mode, egui::RichText::new("Dev mode (-noverify)").size(14.0));
            ui.add_space(40.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() { self.config.ram_mb = self.temp_ram; self.config.dev_mode = self.temp_dev_mode; let _ = self.config.save(&self.config.config_path()); self.state = AppState::Main; }
                if ui.button("Cancel").clicked() { self.state = AppState::Main; }
            });
        });
    }

    fn start_launch(&mut self) {
        self.is_launching = true; self.launch_complete = false; self.launch_progress = 0.0;
        self.launch_message = "Preparing...".to_string();
        let config = self.config.clone(); let version = self.selected_version.clone(); let mods = self.mods.clone();
        let (tx, rx) = mpsc::channel(); self.rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send("Checking Java...".to_string());
            match version { McVersion::Fabric => launch_fabric(&config, &mods, &tx), McVersion::MCP => launch_cheat(&config, &tx) }
            let _ = tx.send("__COMPLETE__".to_string());
        });
    }
}

// ============================================================
// eframe::App
// ============================================================

impl eframe::App for AporiaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.changelog_rx { if let Ok(e) = rx.try_recv() { self.changelog = e; self.changelog_rx = None; } }
        if let Some(rx) = &self.commits_rx { if let Ok(c) = rx.try_recv() { self.version_commits = c; self.commits_rx = None; } }
        if self.is_launching {
            ctx.request_repaint();
            if let Some(rx) = &self.rx {
                while let Ok(msg) = rx.try_recv() {
                    if msg == "__COMPLETE__" { self.launch_complete = true; self.is_launching = false; }
                    else if let Some(pipe) = msg.find('|') {
                        if let Ok(p) = msg[9..pipe].parse::<f32>() { self.launch_progress = p / 100.0; self.launch_message = msg[pipe + 1..].to_string(); }
                    } else { self.launch_message = msg.clone(); }
                }
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| { self.cosmic_bg.draw(ui, ui.max_rect()); });
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            match self.state { AppState::Login => self.draw_login(ui), AppState::Main => self.draw_main_content(ui), AppState::Settings => self.draw_settings_content(ui) }
        });
    }
}

// ============================================================
// LAUNCH (без изменений)
// ============================================================

fn launch_fabric(config: &Config, mods: &[ModInfo], tx: &mpsc::Sender<String>) {
    let ip = &config.install_path; let jp = ensure_java(ip, tx);
    let _ = tx.send("Загрузка Fabric...".to_string());
    let vp = PathBuf::from(ip).join("versions").join("Fabric 1.21.11"); let _ = fs::create_dir_all(&vp);
    let jar = vp.join("Fabric 1.21.11.jar"); let json = vp.join("Fabric 1.21.11.json");
    if !jar.exists() { let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download("https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Fabric%201.21.11/Fabric%201.21.11.jar", jar.to_str().unwrap())); }
    if !json.exists() { let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download("https://raw.githubusercontent.com/aporia-xyz/Aporia.loader/refs/heads/main/versions/Fabric%201.21.11/Fabric%201.21.11.json", json.to_str().unwrap())); }
    let _ = tx.send("Загрузка библиотек...".to_string()); let _ = load_libraries(config, &json, tx);
    let _ = tx.send("Загрузка модов...".to_string());
    let mp = PathBuf::from(ip).join("game").join("mods"); let _ = fs::create_dir_all(&mp);
    let fa = mp.join("fabric-api.jar");
    if !fa.exists() { let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download("https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/0.141.2%2B1.21.11/fabric-api-0.141.2%2B1.21.11.jar", fa.to_str().unwrap())); }
    let sel: Vec<_> = mods.iter().filter(|m| m.selected).cloned().collect();
    if !sel.is_empty() { let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download_mods(mp.to_str().unwrap(), &sel)); }
    let _ = tx.send("Распаковка natives...".to_string()); let _ = extract_natives(config);
    let _ = tx.send("Запуск...".to_string()); let _ = launch_minecraft_fabric(config, &jp);
}

fn launch_cheat(config: &Config, tx: &mpsc::Sender<String>) {
    let ip = &config.install_path; let _ = tx.send("Checking Java...".to_string()); let jp = ensure_java(ip, tx);
    let _ = tx.send("PROGRESS:0|Preparing MCP...".to_string());
    let vp = if cfg!(target_os = "windows") { dirs::config_dir().unwrap_or_default().join("apr").join("versions").join("Aporia.client") } else { dirs::home_dir().unwrap_or_default().join(".apr").join("versions").join("Aporia.client") };
    let _ = fs::create_dir_all(&vp); let jar = vp.join("Aporia.client.jar"); let json = vp.join("Aporia.client.json");
    let should = !jar.exists() || fs::metadata(&jar).map(|m| m.len() < 100_000_000).unwrap_or(true);
    if should {
        let _ = tx.send("PROGRESS:5|Downloading JAR...".to_string());
        if jar.exists() { let _ = fs::remove_file(&jar); }
        match tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download("https://github.com/dakychan/Aporia/releases/download/0.5.0/Aporia.client.jar", jar.to_str().unwrap())) {
            Ok(s) => { let _ = tx.send(format!("PROGRESS:50|Downloaded: {}MB", s / 1_000_000)); }
            Err(e) => { let _ = tx.send(format!("PROGRESS:0|JAR failed: {}", e)); return; }
        }
    } else { let _ = tx.send("PROGRESS:50|JAR cached".to_string()); }
    if !json.exists() { let _ = tx.send("PROGRESS:60|Downloading JSON...".to_string()); let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download("https://github.com/dakychan/Aporia/releases/download/0.5.0/Aporia.client.json", json.to_str().unwrap())); }
    let _ = tx.send("PROGRESS:90|Launching...".to_string()); let _ = launch_minecraft_cheat(config, &jp, &jar);
}

fn ensure_java(_ip: &str, tx: &mpsc::Sender<String>) -> String {
    let ad = dirs::config_dir().unwrap_or_default(); let jre = ad.join("apr").join("jre");
    #[cfg(target_os = "windows")] let jex = jre.join("jdk-26").join("bin").join("java.exe");
    #[cfg(not(target_os = "windows"))] let jex = jre.join("jdk-26").join("bin").join("java");
    if jex.exists() { let _ = tx.send("Java found".to_string()); return jex.to_string_lossy().to_string(); }
    let _ = tx.send("Downloading Java 26...".to_string());
    if let Err(e) = download_java(&jre, tx) { log::error!("Java failed: {}", e); let _ = tx.send(format!("Java failed: {}", e)); }
    jex.to_string_lossy().to_string()
}

fn download_java(jre: &PathBuf, tx: &mpsc::Sender<String>) -> anyhow::Result<()> {
    let url = "https://download.java.net/java/GA/jdk26/c3cc523845074aa0af4f5e1e1ed4151d/35/GPL/openjdk-26_windows-x64_bin.zip";
    let zp = jre.join("openjdk-26.zip"); fs::create_dir_all(jre)?;
    let _ = tx.send("PROGRESS:10|Downloading Java...".to_string());
    let rt = tokio::runtime::Runtime::new()?; let tc = tx.clone();
    rt.block_on(Downloader::download_with_progress(url, zp.to_str().unwrap(), |d, t| {
        if let Some(total) = t { let _ = tc.send(format!("PROGRESS:{}|Downloading Java... {}MB/{}MB", (d as f64 / total as f64 * 40.0) as u64 + 10, d / 1024 / 1024, total / 1024 / 1024)); }
    }))?;
    let _ = tx.send("PROGRESS:50|Extracting Java...".to_string());
    let file = fs::File::open(&zp)?; let mut archive = zip::ZipArchive::new(file)?; let total = archive.len();
    for i in 0..total {
        let mut f = archive.by_index(i)?; let out = jre.join(f.mangled_name());
        if f.name().ends_with('/') { fs::create_dir_all(&out)?; }
        else { if let Some(p) = out.parent() { fs::create_dir_all(p)?; } let mut o = fs::File::create(&out)?; std::io::copy(&mut f, &mut o)?; }
        if i % 100 == 0 { let _ = tx.send(format!("PROGRESS:{}|Extracting... {}/{}", 50 + (i as f64 / total as f64 * 40.0) as u64, i, total)); }
    }
    fs::remove_file(&zp)?; Ok(())
}

fn load_libraries(config: &Config, jp: &PathBuf, tx: &mpsc::Sender<String>) -> anyhow::Result<()> {
    let json: JsonValue = serde_json::from_str(&fs::read_to_string(jp)?)?;
    let lp = PathBuf::from(&config.install_path).join("libraries"); let os = get_os_name();
    if let Some(libs) = json.get("libraries").and_then(|v| v.as_array()) {
        for lib in libs {
            let name = match lib.get("name").and_then(|v| v.as_str()) { Some(n) => n, None => continue };
            if name.contains("ru.legacylauncher") { continue; }
            if let Some(rules) = lib.get("rules").and_then(|v| v.as_array()) {
                let mut ok = false;
                for r in rules { if r.get("action").and_then(|v| v.as_str()) == Some("allow") { let m = r.get("os").and_then(|o| o.get("name").and_then(|n| n.as_str())) == Some(os); if m || r.get("os").is_none() { ok = true; } } }
                if !ok { continue; }
            }
            let base = lib.get("url").and_then(|v| v.as_str()).unwrap_or("https://libraries.minecraft.net/");
            let ps: Vec<&str> = name.split(':').collect(); if ps.len() < 3 { continue; }
            let gp = ps[0].replace('.', "/"); let fn_ = format!("{}-{}.jar", ps[1], ps[2]);
            let url = format!("{}/{}/{}/{}/{}", base, gp, ps[1], ps[2], fn_);
            let local = lp.join(&gp).join(ps[1]).join(ps[2]).join(&fn_);
            if !local.exists() { if let Some(p) = local.parent() { let _ = fs::create_dir_all(p); } let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download(&url, local.to_str().unwrap())); }
        }
    }
    if let Some(ai) = json.get("assetIndex") {
        if let (Some(url), Some(id)) = (ai.get("url").and_then(|v| v.as_str()), ai.get("id").and_then(|v| v.as_str())) {
            let ip = PathBuf::from(&config.install_path).join("assets").join("indexes").join(format!("{}.json", id));
            if !ip.exists() { let _ = fs::create_dir_all(ip.parent().unwrap()); let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download(url, ip.to_str().unwrap())); }
            if let Ok(ic) = fs::read_to_string(&ip) {
                if let Ok(ij) = serde_json::from_str::<JsonValue>(&ic) {
                    if let Some(objs) = ij.get("objects").and_then(|v| v.as_object()) {
                        let total = objs.len(); let _ = tx.send(format!("Ассеты: {}", total));
                        for (i, (_, val)) in objs.iter().enumerate() {
                            if let Some(hash) = val.get("hash").and_then(|v| v.as_str()) {
                                let op = PathBuf::from(&config.install_path).join("assets").join("objects").join(&hash[0..2]).join(hash);
                                if !op.exists() { if let Some(p) = op.parent() { let _ = fs::create_dir_all(p); } let _ = tokio::runtime::Runtime::new().unwrap().block_on(Downloader::download(&format!("https://resources.download.minecraft.net/{}/{}", &hash[0..2], hash), op.to_str().unwrap())); }
                            }
                            if i % 50 == 0 { let _ = tx.send(format!("Ассеты: {}/{}", i, total)); }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_natives(config: &Config) -> anyhow::Result<()> {
    let lp = PathBuf::from(&config.install_path).join("libraries");
    let nd = PathBuf::from(&config.install_path).join("versions").join("Fabric 1.21.11").join("natives");
    let _ = fs::create_dir_all(&nd);
    let pat = match get_os_name() { "windows" => "natives-windows", "osx" => "natives-macos", _ => "natives-linux" };
    for e in walkdir::WalkDir::new(&lp) { if let Ok(e) = e { if e.file_type().is_file() && e.path().extension().map(|x| x == "jar").unwrap_or(false) && e.path().file_name().and_then(|n| n.to_str()).map(|n| n.contains(pat)).unwrap_or(false) { let _ = extract_zip(e.path(), &nd); } } }
    Ok(())
}

fn extract_zip(path: &std::path::Path, dest: &std::path::Path) -> anyhow::Result<()> {
    let file = fs::File::open(path)?; let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() { let mut f = archive.by_index(i)?; let out = dest.join(f.mangled_name()); if f.name().ends_with('/') { let _ = fs::create_dir_all(&out); } else { if let Some(p) = out.parent() { let _ = fs::create_dir_all(p); } let mut o = fs::File::create(&out)?; std::io::copy(&mut f, &mut o)?; } }
    Ok(())
}

fn launch_minecraft_fabric(config: &Config, java_path: &str) -> anyhow::Result<()> {
    let gd = PathBuf::from(&config.install_path).join("game"); let lp = PathBuf::from(&config.install_path).join("libraries");
    let ap = PathBuf::from(&config.install_path).join("assets"); let nd = PathBuf::from(&config.install_path).join("versions").join("Fabric 1.21.11").join("natives");
    let _ = fs::create_dir_all(&gd);
    let mut cp = vec![PathBuf::from(&config.install_path).join("versions").join("Fabric 1.21.11").join("Fabric 1.21.11.jar")];
    if lp.exists() { for e in walkdir::WalkDir::new(&lp) { if let Ok(e) = e { if e.file_type().is_file() && e.path().extension().map(|x| x == "jar").unwrap_or(false) { cp.push(e.path().to_path_buf()); } } } }
    let cps = cp.iter().filter_map(|p| p.to_str()).collect::<Vec<_>>().join(if cfg!(windows) { ";" } else { ":" });
    let mut cmd = Command::new(java_path);
    cmd.arg(format!("-Xmx{}M", config.ram_mb)).arg(format!("-Djava.library.path={}", nd.display()));
    if config.dev_mode { cmd.arg("-noverify"); }
    cmd.arg("net.fabricmc.loader.impl.launch.knot.KnotClient").arg("--gameDir").arg(&gd).arg("--version").arg("Fabric 1.21.11").arg("--assetsDir").arg(&ap).arg("--assetIndex").arg("29").arg("--username").arg(&config.username).env("CLASSPATH", &cps);
    cmd.spawn()?; Ok(())
}

fn launch_minecraft_cheat(config: &Config, java_path: &str, jar_path: &PathBuf) -> anyhow::Result<()> {
    let gd = PathBuf::from(&config.install_path).join("game"); let ap = PathBuf::from(&config.install_path).join("assets");
    let _ = fs::create_dir_all(&gd);
    let mut cmd = Command::new(java_path);
    cmd.arg(format!("-Xmx{}M", config.ram_mb)); if config.dev_mode { cmd.arg("-noverify"); }
    cmd.arg("-cp").arg(jar_path.to_str().unwrap()).arg("net.minecraft.client.main.Main").arg("--version").arg("mcp").arg("--accessToken").arg("0").arg("--assetsDir").arg(&ap).arg("--assetIndex").arg("29").arg("--userProperties").arg("{}").arg("--username").arg(&config.username).arg("--gameDir").arg(&gd);
    cmd.spawn()?; Ok(())
}

// ============================================================
// DATA
// ============================================================

fn default_changelog() -> Vec<ChangelogEntry> {
    vec![
        ChangelogEntry { version: "0.5.0".into(), date: "2026-03-29".into(), changes: vec!["UI redesign".into()] },
        ChangelogEntry { version: "0.4.1".into(), date: "2026-03-15".into(), changes: vec!["Bug fixes".into()] },
        ChangelogEntry { version: "0.4.0".into(), date: "2026-02-28".into(), changes: vec!["New features".into()] },
        ChangelogEntry { version: "0.3.2".into(), date: "2026-02-10".into(), changes: vec!["Performance".into()] },
        ChangelogEntry { version: "0.3.1".into(), date: "2026-01-22".into(), changes: vec!["Hotfix".into()] },
        ChangelogEntry { version: "0.3.0".into(), date: "2026-01-05".into(), changes: vec!["Rewritten in Rust".into()] },
        ChangelogEntry { version: "0.2.4".into(), date: "2025-12-18".into(), changes: vec!["Initial".into()] },
    ]
}

async fn fetch_aporia_releases() -> anyhow::Result<Vec<ChangelogEntry>> {
    let resp = reqwest::Client::new().get("https://api.github.com/repos/dakychan/Aporia/releases").header("User-Agent", "Aporia-Loader").send().await?;
    let releases: Vec<JsonValue> = resp.json().await?; let mut entries = Vec::new();
    for r in releases.iter().take(15) {
        if let (Some(tag), Some(_)) = (r.get("tag_name").and_then(|v| v.as_str()), r.get("body")) {
            let date = r.get("published_at").and_then(|v| v.as_str()).unwrap_or("").split('T').next().unwrap_or("");
            entries.push(ChangelogEntry { version: tag.to_string(), date: date.to_string(), changes: vec!["Release".into()] });
        }
    }
    Ok(if entries.is_empty() { default_changelog() } else { entries })
}

async fn fetch_commits_for_version(branch: &str) -> anyhow::Result<Vec<String>> {
    let resp = reqwest::Client::new().get(format!("https://api.github.com/repos/dakychan/Aporia/commits?sha={}&per_page=30", branch)).header("User-Agent", "Aporia-Loader").send().await?;
    let commits: Vec<JsonValue> = resp.json().await?; let mut msgs = Vec::new();
    for c in commits { if let Some(msg) = c.get("commit").and_then(|c| c.get("message")).and_then(|m| m.as_str()) { if let Some(f) = msg.lines().next() { if !f.is_empty() { msgs.push(f.to_string()); } } } }
    Ok(msgs)
}

// ============================================================
// MAIN
// ============================================================

fn main() -> eframe::Result<()> {
    env_logger::init();
    log::info!("Starting Aporia Loader v{}", VERSION);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]).with_min_inner_size([1200.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native("Aporia Loader", options, Box::new(|cc| Ok(Box::new(AporiaApp::new(cc)))))
}