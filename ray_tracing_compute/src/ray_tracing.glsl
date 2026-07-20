#version 460

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

//
// Constants
//
                
const float FLT_MAX = 100000000.0;
const float PI = 3.1415926535897932385;

//
// Uniform Buffers
//

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
                
//
// Material Buffers
//

struct MetalMat {
    vec3 albedo;
    float fuzz;
};

layout(set = 0, binding = 1) buffer readonly LambertianMaterials {
    vec3 mats[];
} lambertian;

layout(set = 0, binding = 2) buffer readonly MetalMaterials {
    MetalMat mats[];
} metal;

layout(set = 0, binding = 3) buffer readonly DielectricMaterials {
    float mats[];
} dielectric;

//
// Object Buffers
//

struct Material {
    uint mat;
    uint index;
};
// Types:
// Lambertian : 0
// Metal      : 1
// Dielectric : 2

struct Sphere {
    vec4 center_rad;
    Material mat;
};

layout(set = 0, binding = 4, std430) buffer readonly Spheres {
    Sphere spheres[];
} spheres;

// 
// Render Quality Uniforms
//

// TODO: Should probably be specialized constants 
layout(set = 0, binding = 5) uniform readonly QualityParameters {
    uint samples_per_pixel;
    int max_depth;
    uint sphere_count;
} quality;

float pixel_samples_scale = 1.0 / float(quality.samples_per_pixel);

//
// Camera Uniforms
//

layout(set = 0, binding = 6) uniform readonly CameraSettings {
    vec3 look_from;
    vec3 look_at;
    vec3 v_up;
    float image_width;
    float image_height;
    float vfov;
    float defocus_angle;
    float focus_dist;
} camera;

// Image - TODO: Move to Uniform (If we need image size outside of the camera?)

// Camera - TODO: Move to Uniform
float theta = camera.vfov * PI / 180.0;
float h = tan(theta / 2);
float viewport_height = 2.0 * h * camera.focus_dist;
float viewport_width = viewport_height * (camera.image_width / camera.image_height);

// Unit basis vectors
vec3 w = normalize(camera.look_from - camera.look_at);
vec3 u = normalize(cross(camera.v_up, w));
vec3 v = cross(w, u);

// viewport vectors
vec3 viewport_u = viewport_width * u;
vec3 viewport_v = viewport_height * -v;

// pixel deltas
vec3 pixel_delta_u = viewport_u / camera.image_width;
vec3 pixel_delta_v = viewport_v / camera.image_height;

// upper left pixel
vec3 viewport_upper_left = camera.look_from 
                - (camera.focus_dist * w) - viewport_u / 2 - viewport_v / 2;
vec3 pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

// Defocus basis vectors
float defocus_radius = camera.focus_dist * tan((camera.defocus_angle * 0.5) * PI / 180.0);
vec3 defocus_disk_u = u * defocus_radius;
vec3 defocus_disk_v = v * defocus_radius;

//
// Utilities
//

uint current_rand_offset = 0; // Idea from https://github.com/TwentyFiveSoftware/ray-tracing-gpu/tree/master

// https://www.shadertoy.com/view/XlGcRh
// Shadertoy: Hash Functions for GPU Rendering by markjarzynski
// https://www.pcg-random.org/
uint pcg(uint v)
{
    uint state = v * 747796405u + 2891336453u;
    uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// http://www.jcgt.org/published/0009/03/02/
uvec3 pcg3d(uvec3 v) {

    v = v * 1664525u + 1013904223u;

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    v ^= v >> 16u;

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    return v;
}

uvec3 seed() {
    const uvec3 v = floatBitsToUint(vec3(gl_GlobalInvocationID.xy, current_rand_offset));
    current_rand_offset += 1;
    if (current_rand_offset >= 0xFFFFFFF0u) {
        current_rand_offset = 0;
    }
    return v;
}

float rand_float() {
    uvec3 seed = seed();
    uint hash = pcg(pcg(pcg(seed.x) + seed.y) + seed.z);
    return float(hash) * (1.0/float(0xffffffffu));
}

float rand_float_range(float min, float max) {
    float rand = rand_float();
    return min + (max - min) * rand;
}

vec3 rand_vec3() {
    uvec3 hash = pcg3d(seed());
    return vec3(hash)  * (1.0/float(0xffffffffu));
}

vec3 rand_vec3_range(float min, float max) {
    vec3 rand = rand_vec3();
    return min + (max - min) * rand;                
}

vec3 rand_vec3_unit() {
    vec3 p = rand_vec3_range(-1, 1);
    return normalize(p);
}

vec3 rand_vec3_hemisphere(vec3 normal) {
    vec3 unit = rand_vec3_unit();
    if (dot(unit, normal) > 0.0) {
        return unit;
    } else {
        return -unit;
    }
}

vec3 rand_vec3_unit_disk() {
    return vec3(rand_float_range(-1, 1), rand_float_range(-1, 1), 0);
}
                
struct Ray {
    vec3 orig;
    vec3 dir;
};

vec3 ray_at(Ray r, float t) {
    return (r.dir * t) + r.orig;
}

bool get_front_face(Ray r, vec3 out_norm) {
    return dot(r.dir, out_norm) < 0;
}

struct HitRecord {
    vec3 p;
    vec3 normal;
    Material mat;
    float t;
    bool front_face;
    bool hit;
};

HitRecord new_hit_record() {
    HitRecord rec;
    rec.p = vec3(0);
    rec.normal = vec3(0);
    rec.mat = Material(0, 0);
    rec.t = 0.0;
    rec.front_face = true;
    rec.hit = false;
    return rec;
}

bool near_zero(vec3 v) {
    float EPSILON = 1e-8;
    return abs(v.x) < EPSILON && abs(v.y) < EPSILON && abs(v.z) < EPSILON;
}

//
// Materials
//

struct MaterialScatter {
    Ray ray;
    vec3 attenuation;
    bool scattered;
};

MaterialScatter lambertian_scatter(Ray r, HitRecord rec) {
    vec3 albedo = lambertian.mats[rec.mat.index];
    MaterialScatter ret;
    vec3 dir = rec.normal + rand_vec3_unit();
    if (near_zero(dir)) {
        dir = rec.normal;
    }
    ret.ray = Ray(rec.p, dir);
    ret.attenuation = albedo;
    ret.scattered = true;
    return ret;
}

vec3 reflect_vec3(vec3 v, vec3 n) {
    return v - (2.0 * dot(v, n) * n);
}

MaterialScatter metal_scatter(Ray r, HitRecord rec) {
    MetalMat mat = metal.mats[rec.mat.index];
    MaterialScatter ret;
    vec3 dir = reflect_vec3(r.dir, rec.normal);
    dir = normalize(dir) + (mat.fuzz * rand_vec3_unit());
    ret.ray = Ray(rec.p, dir);
    ret.attenuation = mat.albedo;
    ret.scattered = true;
    return ret;
}

vec3 refract_vec3(vec3 uv, vec3 n, float etai_over_etat) {
    float cos_theta = min(dot(-uv, n), 1.0);
    vec3 r_out_perp = etai_over_etat * (uv + cos_theta * n);
    vec3 r_out_parallel = -sqrt(abs(1.0 - dot(r_out_perp, r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}

float reflectance(float cosine, float refraction_index) {
    float r0 = (1 - refraction_index) / (1 + refraction_index);
    r0 = r0 * r0;
    return r0 + (1 - r0) * pow((1 - cosine), 5);
}

MaterialScatter dielectric_scatter(Ray r, HitRecord rec) {
    float refraction_index = dielectric.mats[rec.mat.index];
    float ri = rec.front_face ? (1.0 / refraction_index) : refraction_index;

    vec3 unit_dir = normalize(r.dir);
    float cos_theta = min(dot(-unit_dir, rec.normal), 1.0);
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    bool cannot_refract = ri * sin_theta > 1.0;
    vec3 dir;

    if (cannot_refract || reflectance(cos_theta, ri) > rand_float()) {
        dir = reflect_vec3(unit_dir, rec.normal);
    } else {
        dir = refract_vec3(unit_dir, rec.normal, ri);
    }

    MaterialScatter ret;
    ret.attenuation = vec3(1);
    ret.ray = Ray(rec.p, dir);
    ret.scattered = true;
    return ret;
}

//
// Intersection methods
//

struct Interval {
    float min;
    float max;
};

bool interval_surrounds(Interval bounds, float val) {
    return bounds.min < val && val < bounds.max;
}

float interval_clamp(Interval bounds, float val) {
    if (val < bounds.min) {
        return bounds.min;
    }
    if (val > bounds.max) {
        return bounds.max;
    }
    return val;
}

HitRecord hit_sphere(Sphere s, Ray r, Interval ray_t) {
    vec3 oc = s.center_rad.xyz - r.orig;
    float a = dot(r.dir, r.dir);
    float h = dot(r.dir, oc);
    float c = dot(oc, oc) - s.center_rad.w * s.center_rad.w;
    float discriminant = h*h - a*c;

    HitRecord rec = new_hit_record();
    if (discriminant < 0) {
        return rec;
    }

    float sqrtd = sqrt(discriminant);

    float root = (h - sqrtd) / a;
    if (!interval_surrounds(ray_t, root)) {
        root = (h + sqrtd) / a;
        if (!interval_surrounds(ray_t, root)) {
            return rec;
        }
    }

    rec.t = root;
    rec.p = ray_at(r, root);

    vec3 out_norm = (rec.p - s.center_rad.xyz) / s.center_rad.w;
    rec.front_face = get_front_face(r, out_norm);
    rec.normal = rec.front_face ? out_norm : -out_norm;
    rec.mat = s.mat;
    rec.hit = true;

    return rec;
} 

HitRecord hit_world(Ray r, Interval ray_t) {
    HitRecord rec = new_hit_record();
    float closest = ray_t.max;

    for (int iter = 0; iter < quality.sphere_count; iter++) {
        HitRecord temp_rec = hit_sphere(spheres.spheres[iter], r, Interval(ray_t.min, closest));
        if (temp_rec.hit) {
            closest = temp_rec.t;
            rec = temp_rec;
        }
    }

    return rec;
}

//
// Rendering
//

vec2 sample_square() {
    return vec2(rand_float() - 0.5, rand_float() - 0.5);
}

vec3 defocus_disk_sample() {
    vec2 p = vec2(0);
    for (int iter = 0; iter < 10; iter++) {
        vec2 temp = vec2(rand_float_range(-1, 1), rand_float_range(-1, 1));
        if (dot(temp, temp) < 1) {
            p = temp;
            break;
        }
    }
    return camera.look_from + (p.x * defocus_disk_u) + (p.y * defocus_disk_v);
}

Ray get_ray(vec2 loc) {
    vec2 offset = sample_square();
    vec3 pixel_center = pixel00_loc
        + ((loc.x + offset.x) * pixel_delta_u)
        + ((loc.y + offset.y) * pixel_delta_v);
    vec3 ray_origin = (camera.defocus_angle <= 0) ? camera.look_from : defocus_disk_sample();
    vec3 ray_direction = pixel_center - ray_origin;
    Ray r = Ray(ray_origin, ray_direction);
    return r;
}

vec3 ray_color(Ray r, int depth) {
    vec3 color_mult = vec3(1);
    for (int iter = 0; iter < depth; iter++) {
        HitRecord rec = hit_world(r, Interval(0.001, FLT_MAX));
        if (rec.hit) {
            // Scatter
            MaterialScatter scat;
            if (rec.mat.mat == 0) {
                scat = lambertian_scatter(r, rec);
            } else if (rec.mat.mat == 1) {
                scat = metal_scatter(r, rec);
            } else if (rec.mat.mat == 2) {
                scat = dielectric_scatter(r, rec);
            }

            if (scat.scattered) {
                r = scat.ray;
                color_mult *= scat.attenuation;
            } else {
                break;
            }
        } else {
            vec3 unit_direction = normalize(r.dir);
            float a = 0.5 * (unit_direction.y + 1.0);
            return color_mult * ((1.0 - a) * vec3(1.0) + a * vec3(0.5, 0.7, 1.0));
        }
    }
    return vec3(0);
}

float linear_to_gamma(float linear) {
    if (linear > 0) {
        return sqrt(linear);
    }
    return 0;
}

vec3 vec_to_color(vec3 color) {
    vec3 ret_color = color;
    ret_color.x = linear_to_gamma(ret_color.x);
    ret_color.y = linear_to_gamma(ret_color.y);
    ret_color.z = linear_to_gamma(ret_color.z);

    Interval intensity = Interval(0, 0.999);
    ret_color.x = interval_clamp(intensity, ret_color.x);
    ret_color.y = interval_clamp(intensity, ret_color.y);
    ret_color.z = interval_clamp(intensity, ret_color.z);

    return ret_color;
}

void main() {
    vec3 pixel_color = vec3(0, 0, 0);
    for (int iter = 0; iter < quality.samples_per_pixel; iter++) {
        Ray r = get_ray(gl_GlobalInvocationID.xy);
        pixel_color += ray_color(r, quality.max_depth);
    }

    vec4 to_write = vec4(vec_to_color(pixel_color * pixel_samples_scale), 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}