#version 330 core

in vec2 v_pos;
out vec4 FragColor;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform float u_blur_radius;

void main() {
    vec2 uv = v_pos / u_resolution;
    vec4 color = vec4(0.0);
    float total_weight = 0.0;
    
    // Gaussian blur
    for (float x = -u_blur_radius; x <= u_blur_radius; x += 1.0) {
        for (float y = -u_blur_radius; y <= u_blur_radius; y += 1.0) {
            float distance = sqrt(x * x + y * y);
            float weight = exp(-distance * distance / (2.0 * u_blur_radius * u_blur_radius));
            
            vec2 offset = vec2(x, y) / u_resolution;
            color += texture(u_texture, uv + offset) * weight;
            total_weight += weight;
        }
    }
    
    FragColor = color / total_weight;
}
