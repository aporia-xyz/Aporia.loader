struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    rect_size: vec2<f32>,
    corner_radius: f32,
    border_radius: f32,
    edge_thickness: f32,
    padding: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var u_sampler: sampler;

@group(0) @binding(2)
var u_texture: texture_2d<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

// SDF для закругленного прямоугольника
fn rounded_rect_sdf(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let half_size = size * 0.5;
    let corner = abs(p) - half_size + vec2<f32>(radius);
    let corner_max = max(corner, vec2<f32>(0.0));
    let corner_len = length(corner_max);
    return min(max(corner.x, corner.y), 0.0) + corner_len - radius;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Преобразуем UV в диапазон [-1, 1] для SDF
    let uv_centered = input.uv * 2.0 - 1.0;
    
    // Вычисляем расстояние до края
    let dist = rounded_rect_sdf(
        uv_centered, 
        uniforms.rect_size, 
        uniforms.corner_radius
    );
    
    // Сглаживание краев
    let smoothing = 1.0;
    let alpha = 1.0 - smoothstep(-smoothing, smoothing, dist);
    
    // Цвет текстуры
    let tex_color = textureSample(u_texture, u_sampler, input.uv);
    
    // Добавляем обводку
    let border_dist = rounded_rect_sdf(
        uv_centered, 
        uniforms.rect_size - uniforms.border_radius, 
        uniforms.corner_radius - uniforms.border_radius
    );
    
    let border_alpha = 1.0 - smoothstep(-smoothing, smoothing, border_dist);
    let is_border = border_alpha > 0.5 && alpha > 0.5;
    
    var final_color = tex_color;
    
    if is_border {
        final_color = vec4<f32>(1.0, 0.5, 0.0, 1.0); // Цвет обводки
    }
    
    return vec4<f32>(final_color.rgb, alpha);
}
