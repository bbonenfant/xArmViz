// shader.vert
#version 450

struct Light {
    vec3 position;
    vec3 color;
    mat4 projection;
};

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;
layout(location=2) in vec3 a_normal;
layout(location=3) in mat4 a_model_matrix;
layout(location=7) in mat3 a_normal_matrix;

layout(location=0) out vec3 v_position;
layout(location=1) out vec2 v_tex_coords;
layout(location=2) out vec3 v_normal;
layout(location=3) out vec4 v_position_light[10];

layout(set=1, binding=0)
uniform Uniforms {
    vec3 u_view_position;
    mat4 u_view_proj;
};

layout(set=2, binding=0) 
uniform LightCount {
    uint u_LightCount;
};
layout(set=2, binding=1) 
uniform Lights {
    Light u_Lights[10];
};


void main() {
    vec4 model_space = a_model_matrix * vec4(a_position, 1.0);
    v_position = model_space.xyz;
    v_tex_coords = a_tex_coords;
    v_normal = a_normal_matrix * a_normal;

    for (int i=0; i<10; ++i) {
        if (i == u_LightCount) { break; }
        Light light = u_Lights[i];
        v_position_light[i] = light.projection * vec4(v_position, 1.0);
    }

    gl_Position = u_view_proj * model_space;
}