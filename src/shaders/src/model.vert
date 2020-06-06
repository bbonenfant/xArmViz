// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;
layout(location=2) in vec3 a_normal;
layout(location=3) in mat4 a_model_matrix;
layout(location=7) in mat3 a_normal_matrix;

layout(location=0) out vec3 v_position;
layout(location=1) out vec2 v_tex_coords;
layout(location=2) out vec3 v_normal;

layout(set=1, binding=0)
uniform Uniforms {
    vec3 u_view_position;
    mat4 u_view_proj;
};


void main() {
    vec4 model_space = a_model_matrix * vec4(a_position, 1.0);
    v_position = model_space.xyz;
    v_tex_coords = a_tex_coords;
    v_normal = a_normal_matrix * a_normal;

    gl_Position = u_view_proj * model_space;
}