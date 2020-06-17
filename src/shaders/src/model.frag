// shader.frag
#version 450

const uint MAX_LIGHTS = 10;

struct Light {
    vec3 position;
    vec3 color;
    mat4 projection;
};

layout(location=0) in vec3 v_position;
layout(location=1) in vec2 v_tex_coords;
layout(location=2) in vec3 v_normal;
layout(location=3) in vec4 v_position_light[MAX_LIGHTS];

layout(location=0) out vec4 f_color;

layout(set=0, binding=0) uniform texture2D t_diffuse;
layout(set=0, binding=1) uniform sampler s_diffuse;
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
    Light u_Lights[MAX_LIGHTS];
};
layout(set=3, binding=0) uniform texture2DArray t_shadow;
layout(set=3, binding=1) uniform samplerShadow s_shadow;

float compute_shadow(int index) {
    vec4 position_light = v_position_light[index];

    if (position_light.w <= 0.0) {
        return 0.0;
    }

    // compensate for the Y-flip difference between the NDC and texture coordinates
    const vec2 flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    vec4 light_local = vec4(
        position_light.xy * flip_correction/position_light.w + 0.5,
        index,
        position_light.z / position_light.w
    );

    // do the lookup, using HW PCF and comparison
    return texture(sampler2DArrayShadow(t_shadow, s_shadow), light_local);
}

void main() {
    vec3 normal = normalize(v_normal);
    vec3 result = vec3(0.1, 0.1, 0.1);
    for (int i=0; i<MAX_LIGHTS; ++i) {
        if (i == u_LightCount) { break; }
        Light light = u_Lights[i];

        // Compute the diffuse color.
        vec3 light_dir = normalize(light.position - v_position);
        float diffuse_strength = max(dot(normal, light_dir), 0.0);
        vec3 diffuse_color = diffuse_strength * light.color;

        // Comput the specular color.
        vec3 view_dir = normalize(u_view_position - v_position);
        vec3 half_dir = normalize(view_dir + light_dir);
        float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32);
        vec3 specular_color = specular_strength * light.color;

        // Combine the all the colors.
        float shadow = compute_shadow(i);
        result += (shadow * (diffuse_color + specular_color)) / float(u_LightCount);
    }

    vec4 object_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    f_color = vec4(result, 1.0) * object_color;
}
