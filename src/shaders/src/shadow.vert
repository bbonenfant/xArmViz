#version 450
layout(location=0) in vec3 a_position;
layout(location=3) in mat4 a_model_matrix;

layout(set=0, binding=0) 
uniform Light {
    vec3 light_position;
    vec3 light_color;
    mat4 light_view_projection;
};

void main() {
    gl_Position = light_view_projection * a_model_matrix * vec4(a_position, 1.0);
}