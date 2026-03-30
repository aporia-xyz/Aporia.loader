struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    time: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let uv = vec2<f32>(f32(vertex_index & 1u), f32((vertex_index >> 1u) & 1u)) * 2.0;
    let pos = uv * 2.0 - 1.0;
    return VertexOutput(vec4<f32>(pos, 0.0, 1.0), uv);
}

// Улучшенный шум
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 19.19);
    return fract((p3.x + p3.y) * p3.z);
}

fn stars(uv: vec2<f32>) -> f32 {
    let cell_size = 50.0;
    let grid = floor(uv * cell_size);
    let frac = fract(uv * cell_size);
    
    var brightness = 0.0;
    
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            let neighbor = grid + vec2<f32>(f32(x), f32(y));
            let rand = hash21(neighbor);
            
            if (rand > 0.9) {
                let star_x = fract(rand * 12.9898);
                let star_y = fract(rand * 78.233);
                let star_pos = vec2<f32>(star_x, star_y);
                
                let offset = frac - (vec2<f32>(f32(x), f32(y)) + star_pos);
                let dist = length(offset);
                
                let glow = exp(-dist * dist * 300.0) * 0.8;
                brightness = max(brightness, glow);
            }
        }
    }
    
    return brightness;
}

fn comets(uv: vec2<f32>, time: f32) -> vec3<f32> {
    var color = vec3<f32>(0.0);
    
    // 3 кометы
    for (var i: i32 = 0; i < 3; i = i + 1) {
        let comet_seed = f32(i) * 0.333;
        let comet_speed = 0.2 + hash21(vec2<f32>(comet_seed, 0.0)) * 0.1;
        
        // Позиция кометы - прямая линия из правого верха в левый нижний
        let progress = fract(time * comet_speed + comet_seed);
        
        // Начинает справа-сверху (2, 2), летит влево-вниз (0, 0)
        let comet_x = 2.0 - progress * 2.0;
        let comet_y = 2.0 - progress * 2.0;
        let comet_pos = vec2<f32>(comet_x, comet_y);
        
        // Направление движения - всегда влево-вниз (45 градусов)
        let velocity = normalize(vec2<f32>(-1.0, -1.0));
        
        // Расстояние от пикселя до кометы
        let to_comet = uv - comet_pos;
        let dist_to_comet = length(to_comet);
        
        // === КРУГ ===
        let circle_radius = 0.02;
        let circle_glow = exp(-dist_to_comet * dist_to_comet / (circle_radius * circle_radius) * 150.0);
        
        // Белое ядро
        color = color + vec3<f32>(1.0, 1.0, 1.0) * circle_glow * 0.9;
        
        // Фиолетовое свечение вокруг круга
        let outer_glow = exp(-dist_to_comet * dist_to_comet / (circle_radius * 2.5 * circle_radius * 2.5) * 80.0);
        color = color + vec3<f32>(0.8, 0.4, 1.0) * outer_glow * 0.5;
        
        // === ХВОСТ ===
        // Хвост идёт в обратном направлении движения (вверх-вправо)
        let max_tail_length = 0.4;
        let current_tail_length = max_tail_length * progress;  // Хвост растёт по мере движения
        
        // Проекция на линию хвоста
        let proj = dot(to_comet, -velocity);
        let perp = to_comet - proj * (-velocity);
        let dist_perp = length(perp);
        
        // Хвост существует только если мы позади круга
        if (proj > 0.0 && proj < current_tail_length) {
            // Ширина хвоста сужается
            let tail_width = 0.025 * (1.0 - proj / current_tail_length);
            
            // Гауссово размытие хвоста - сильнее
            let tail_glow = exp(-dist_perp * dist_perp / (tail_width * tail_width) * 200.0);
            
            // Затухание: старая часть хвоста темнеет, новая светит
            let tail_age = proj / current_tail_length;  // 0 = новая часть, 1 = старая часть
            let tail_brightness = 1.0 - tail_age * 0.7;  // Старая часть на 70% темнее
            
            // Фиолетовый хвост с динамическим затуханием
            let tail_color = vec3<f32>(0.9, 0.5, 1.0);
            color = color + tail_color * tail_glow * tail_brightness * 2.5;
        }
    }
    
    return color;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let time = uniforms.time;
    
    // Фиолетовый фон
    var color = vec3<f32>(0.024, 0.024, 0.055);
    
    // Добавляем звезды
    let star = stars(uv);
    color = color + vec3<f32>(0.9, 0.85, 1.0) * star * 0.8;
    
    // Добавляем кометы
    color = color + comets(uv, time);
    
    return vec4<f32>(color, 1.0);
}
