#version 450

// The position of the vertex in relation to the center of the model.
layout(location=0) in vec3 a_position;

layout(location=0) out vec3 v_color;

layout(set=0, binding=0)
uniform Uniforms {
    vec3 u_view_position; // unused
    mat4 u_view_proj;
};

layout(set=1, binding=0)
uniform Light {
    vec3 u_position;
    vec3 u_color;
};

// Scale down the size of the light box.
float scale = 0.25;

void main() {
    vec3 v_position = a_position * scale + u_position;
    gl_Position = u_view_proj * vec4(v_position, 1);

    v_color = vec3(0.75, 0.0, 0.0);
}