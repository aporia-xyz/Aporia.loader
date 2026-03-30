//! Космический фон с звездами и кометами

use eframe::egui;
use std::time::Instant;

/// Звезда на фоне
#[derive(Clone)]
struct Star {
    x: f32,
    y: f32,
    radius: f32,
    base_alpha: f32,
    phase: f32,
    speed: f32,
}

/// Комета
#[derive(Clone)]
struct Comet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    tail_len: f32,
    width: f32,
    hue: f32,
    life: f32,
    decay: f32,
}

impl Comet {
    fn new(screen_width: f32, screen_height: f32) -> Self {
        let angle = (rand::random::<f32>() * 20.0 + 30.0).to_radians();
        let speed = rand::random::<f32>() * 6.0 + 5.0;

        Self {
            x: screen_width + rand::random::<f32>() * 400.0,
            y: -rand::random::<f32>() * 300.0,
            vx: -angle.cos() * speed,
            vy: angle.sin() * speed,
            tail_len: rand::random::<f32>() * 80.0 + 100.0,
            width: rand::random::<f32>() * 2.0 + 1.5,
            hue: if rand::random::<f32>() > 0.5 {
                260.0
            } else {
                220.0
            },
            life: 1.0,
            decay: rand::random::<f32>() * 0.002 + 0.0015,
        }
    }

    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt * 60.0;
        self.y += self.vy * dt * 60.0;
        self.life -= self.decay * dt * 60.0;
    }

    fn is_dead(&self, screen_width: f32, screen_height: f32) -> bool {
        self.life <= 0.0 || self.x < -200.0 || self.y > screen_height + 200.0
    }
}

/// Космический фон
pub struct CosmicBackground {
    stars: Vec<Star>,
    comets: Vec<Comet>,
    start_time: Instant,
    last_flash: f32,
    next_flash_at: f32,
    flash_alpha: f32,
}

impl Default for CosmicBackground {
    fn default() -> Self {
        Self::new()
    }
}

impl CosmicBackground {
    pub fn new() -> Self {
        let mut stars = Vec::new();
        for _ in 0..320 {
            stars.push(Star {
                x: rand::random::<f32>() * 3000.0 - 500.0,
                y: rand::random::<f32>() * 2000.0 - 200.0,
                radius: rand::random::<f32>() * 1.4 + 0.3,
                base_alpha: rand::random::<f32>() * 0.6 + 0.2,
                phase: rand::random::<f32>() * std::f32::consts::PI * 2.0,
                speed: rand::random::<f32>() * 0.5 + 0.3,
            });
        }

        Self {
            stars,
            comets: Vec::new(),
            start_time: Instant::now(),
            last_flash: 0.0,
            next_flash_at: rand::random::<f32>() * 3.0 + 2.0,
            flash_alpha: 0.0,
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let dt = ui.ctx().input(|i| i.stable_dt);

        let painter = ui.painter();

        // Фон
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(6, 6, 14));

        // Туман (nebula)
        self.draw_nebula(painter, rect, elapsed);

        // Звезды
        self.draw_stars(painter, rect, elapsed);

        // Кометы
        self.update_and_draw_comets(painter, rect, dt);

        // Вспышки
        self.update_flash(dt);
        if self.flash_alpha > 0.0 {
            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(
                    200,
                    180,
                    255,
                    (self.flash_alpha * 30.0) as u8,
                ),
            );
        }

        // Запрос перерисовки для анимации
        ui.ctx().request_repaint();
    }

    fn draw_nebula(&self, painter: &egui::Painter, rect: egui::Rect, t: f32) {
        let w = rect.width();
        let h = rect.height();

        // Первое пятно
        let nx = w * 0.3 + (t * 0.2).sin() * 80.0;
        let ny = h * 0.4 + (t * 0.15).cos() * 60.0;

        painter.circle_filled(
            rect.min + egui::vec2(nx, ny),
            500.0,
            egui::Color32::from_rgba_unmultiplied(88, 28, 135, 15),
        );

        // Второе пятно
        let nx2 = w * 0.75 + (t * 0.18).cos() * 60.0;
        let ny2 = h * 0.6 + (t * 0.22).sin() * 50.0;

        painter.circle_filled(
            rect.min + egui::vec2(nx2, ny2),
            400.0,
            egui::Color32::from_rgba_unmultiplied(30, 58, 138, 13),
        );
    }

    fn draw_stars(&self, painter: &egui::Painter, rect: egui::Rect, t: f32) {
        let w = rect.width();
        let h = rect.height();

        for star in &self.stars {
            let flicker = (t * star.speed + star.phase).sin() * 0.25;
            let alpha = (star.base_alpha + flicker).max(0.05);

            let x = (star.x % w) + rect.min.x;
            let y = (star.y % h) + rect.min.y;

            painter.circle_filled(
                egui::pos2(x, y),
                star.radius,
                egui::Color32::from_rgba_unmultiplied(220, 215, 255, (alpha * 255.0) as u8),
            );
        }
    }

    fn update_and_draw_comets(&mut self, painter: &egui::Painter, rect: egui::Rect, dt: f32) {
        let w = rect.width();
        let h = rect.height();

        // Создаем кометы если их мало
        while self.comets.len() < 5 {
            self.comets.push(Comet::new(w, h));
        }

        // Обновляем и рисуем
        for i in (0..self.comets.len()).rev() {
            self.comets[i].update(dt);

            if self.comets[i].is_dead(w, h) {
                self.comets[i] = Comet::new(w, h);
            }

            let comet = &self.comets[i];

            // Хвост кометы
            let head = rect.min + egui::vec2(comet.x, comet.y);
            let tail = head - egui::vec2(comet.vx, comet.vy) * comet.tail_len * 0.3;

            // Рисуем хвост как линию с градиентом (упрощенно)
            let alpha = (comet.life * 200.0) as u8;
            painter.line_segment(
                [head, tail],
                egui::Stroke::new(
                    comet.width,
                    egui::Color32::from_rgba_unmultiplied(180, 140, 255, alpha),
                ),
            );

            // Свечение головы
            let glow_radius = comet.width * 6.0 * comet.life;
            if glow_radius > 0.5 {
                painter.circle_filled(
                    head,
                    glow_radius,
                    egui::Color32::from_rgba_unmultiplied(
                        200,
                        160,
                        255,
                        (comet.life * 100.0) as u8,
                    ),
                );
            }
        }
    }

    fn update_flash(&mut self, dt: f32) {
        self.last_flash += dt;

        if self.last_flash > self.next_flash_at {
            self.flash_alpha = 1.0;
            self.last_flash = 0.0;
            self.next_flash_at = rand::random::<f32>() * 5.0 + 3.0;
        }

        if self.flash_alpha > 0.0 {
            self.flash_alpha -= dt * 6.0;
            if self.flash_alpha < 0.0 {
                self.flash_alpha = 0.0;
            }
        }
    }

    pub fn trigger_flash(&mut self) {
        self.flash_alpha = 1.0;
    }
}
