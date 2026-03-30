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

// Простой шум для звезд
fn hash(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.13);
    return fract(p3.x * p3.y * (p3.x + p3.y));
}

fn stars(uv: vec2<f32>) -> f32 {
    let grid = floor(uv * 100.0);
    let frac = fract(uv * 100.0);
    
    var star_brightness = 0.0;
    for (var x: i32 = -1; x <= 1; x = x + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            let neighbor = grid + vec2<f32>(f32(x), f32(y));
            let h = hash(neighbor);
            
            if (h > 0.95) {
                let star_pos = fract(neighbor * 0.1234);
                let dist = distance(frac, star_pos);
                let brightness = exp(-dist * dist * 50.0) * (h - 0.95) * 20.0;
                star_brightness = max(star_brightness, brightness);
            }
        }
    }
    return star_brightness;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    
    // Фиолетовый фон
    var color = vec3<f32>(0.024, 0.024, 0.055);
    
    // Добавляем звезды
    let star = stars(uv);
    color = color + vec3<f32>(0.8, 0.7, 1.0) * star;
    
    return vec4<f32>(color, 1.0);
}
