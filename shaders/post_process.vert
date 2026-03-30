#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;

out vec2 v_pos;
out vec2 v_tex_coord;

uniform mat4 u_projection;

void main() {
    v_pos = position;
    v_tex_coord = tex_coord;
    gl_Position = u_projection * vec4(position, 0.0, 1.0);
}
