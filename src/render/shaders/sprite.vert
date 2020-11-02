#version 450
#extension GL_ARB_separate_shader_objects : enable
layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

// Quad transform.
layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;

layout(location = 2) in mat4 model;
layout(location = 6) in vec4 color;


layout(location = 0) out VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;


void main() {
    vertex.tex_uv = uv;
    vertex.color = color;
     
    vec4 pos4 = model * vec4(pos, 1.0);
    gl_Position =  proj_view * pos4;

    
}
