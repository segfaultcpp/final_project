#version 460

in vec3 frag_position;
in vec3 frag_normal;

uniform vec3 CameraPos;

uniform vec3 Albedo;
uniform float Roughness;
uniform float Metallic;
uniform float Ao;

out vec4 out_color;

const float PI = 3.14159265359;

/// Trowbridge-Reitz GGX
/// Calculates D part of BRDF
/// n: normal vector of the surface (fragment)
/// h: halfway vector
/// a: roughness of the surface (fragment)
float norm_dist(vec3 n, vec3 h, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float n_dot_h2 = pow(max(dot(n, h), 0.0), 2);

    float denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    denom = PI * denom * denom;

    return a2 / denom;
}

float geometry_schlick_ggx(float n_dot, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;
    
    float denom = n_dot * (1.0 - k) + k;
    return n_dot / denom;
}

/// Smith's method
/// Calculates G part of BRDF
/// n: normal vector of the surface (fragment)
/// v: view vector
/// l: light vector of incident light
/// k: roughness remapping
float geometry_func(vec3 n, vec3 v, vec3 l, float a) {
    float n_dot_v = max(dot(n, v), 0.0);
    float n_dot_l = max(dot(n, l), 0.0);

    float ggx1 = geometry_schlick_ggx(n_dot_v, a);
    float ggx2 = geometry_schlick_ggx(n_dot_l, a);

    return ggx1 * ggx2;
}

/// Fresnel-Schlick
/// Calculates F(Fresnel) part of BRDF
vec3 fresnel_schlick(float cos_theta, vec3 f0) {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

void main() {
    vec3 normal = normalize(frag_normal);
    vec3 view = normalize(CameraPos - frag_position);
    vec3 light_dir = vec3(0.0, 0.0, 1.0);

    vec3 f0 = vec3(0.04);
    f0 = mix(f0, Albedo, Metallic);
    
    vec3 h = normalize(view + light_dir);
    vec3 radiance = vec3(1.0);

    float d = norm_dist(normal, h, Roughness);
    float g = geometry_func(normal, view, light_dir, Roughness);
    vec3  f = fresnel_schlick(max(dot(h, view), 0.0), f0);

    vec3 ks = f;
    vec3 kd = vec3(1.0) - ks;
    kd *= 1.0 - Metallic;

    float denom = 4.0 * max(dot(normal, view), 0.0) * max(dot(normal, light_dir), 0.0) + 0.0001;
    vec3 specular = (d * g * f) / denom;

    float n_dot_l = max(dot(normal, light_dir), 0.0);
    vec3 irradiance = (kd * Albedo / PI + specular) * radiance * n_dot_l;

    vec3 ambient = vec3(0.03) * Albedo * Ao;
    vec3 color = ambient + irradiance;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    out_color = vec4(color, 1.0);
}
