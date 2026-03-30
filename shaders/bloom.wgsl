struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

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
    
    // Проверяем текущую ячейку и соседей
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            let neighbor = grid + vec2<f32>(f32(x), f32(y));
            let rand = hash21(neighbor);
            
            // Только 10% ячеек имеют звезду
            if (rand > 0.9) {
                // Случайная позиция звезды в ячейке
                let star_x = fract(rand * 12.9898);
                let star_y = fract(rand * 78.233);
                let star_pos = vec2<f32>(star_x, star_y);
                
                // Расстояние до звезды
                let offset = frac - (vec2<f32>(f32(x), f32(y)) + star_pos);
                let dist = length(offset);
                
                // Острое свечение - меньше размытия
                let glow = exp(-dist * dist * 300.0) * 0.8;
                brightness = max(brightness, glow);
            }
        }
    }
    
    return brightness;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    
    // Фиолетовый фон
    var color = vec3<f32>(0.024, 0.024, 0.055);
    
    // Добавляем звезды
    let star = stars(uv);
    color = color + vec3<f32>(0.9, 0.85, 1.0) * star * 0.8;
    
    return vec4<f32>(color, 1.0);
}
