#version 450
#extension GL_ARB_separate_shader_objects : enable
layout(set = 1, binding = 0) uniform sampler2D u_tex;

layout(location = 0) in VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(u_tex, vertex.tex_uv) * vertex.color;
    out_color = color;
}
