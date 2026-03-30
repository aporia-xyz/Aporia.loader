#version 330 core

in vec2 v_pos;
out vec4 FragColor;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform float u_bloom_strength;

void main() {
    vec2 uv = v_pos / u_resolution;
    
    // Извлекаем яркие пиксели (bloom)
    vec4 color = texture(u_texture, uv);
    float brightness = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    
    if (brightness > 0.5) {
        color.rgb *= u_bloom_strength;
    }
    
    FragColor = color;
}
