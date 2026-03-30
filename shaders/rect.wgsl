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

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    
    // Первый прямоугольник - 75% экрана (левая часть)
    if (uv.x < 1.5) {
        let color = vec3<f32>(0.3, 0.1, 0.5);
        return vec4<f32>(color, 0.4);
    }
    
    // Второй прямоугольник - 25% экрана (правая часть)
    if (uv.x >= 1.5) {
        let color = vec3<f32>(0.2, 0.05, 0.3);
        return vec4<f32>(color, 0.6);
    }
    
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
