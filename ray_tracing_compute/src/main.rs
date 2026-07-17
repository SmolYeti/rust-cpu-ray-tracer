use vulkano::VulkanLibrary;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo,
    allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
};
use vulkano::descriptor_set::{
    DescriptorSet, WriteDescriptorSet,
    allocator::StandardDescriptorSetAllocator,
};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::{
    ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
    compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
};
use vulkano::sync::{self, GpuFuture};

use std::sync::Arc;

use image::{ImageBuffer, Rgba};

// Auto Format:
//      Windows: Shift + Alt + F
//      Mac: Shift + Option + F

// Vulkano Code from: https://vulkano.rs/01-introduction/01-introduction.html
// Ray Tracing code from: https://raytracing.github.io/books/RayTracingInOneWeekend.html

fn main() {
    initalization();
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
                #version 460

                layout(local_size_x = 32, local_size_y = 8, local_size_z = 1) in;

                layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

                //
                // Utilities
                //

                const float FLT_MAX = 3.402823466e+38;
                uint currentRandomOffset = 0;

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
                    currentRandomOffset += 1;
                    const uvec3 v = floatBitsToUint(vec3(gl_GlobalInvocationID.xy, currentRandomOffset));
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

                //
                // Objects
                //
                struct Sphere {
                    vec3 center;
                    float radius;
                };

                const Sphere spheres[] = Sphere[](Sphere(vec3(0, 0, -1), 0.5), Sphere(vec3(0, -100.5, -1), 100.0));
                const int sphere_count = 2;

                //
                // Intersection methods
                //
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

                struct HitRecord {
                    vec3 p;
                    vec3 normal;
                    float t;
                    bool front_face;
                    bool hit;
                };

                HitRecord new_hit_record() {
                    HitRecord rec;
                    rec.p = vec3(0);
                    rec.normal = vec3(0);
                    rec.t = 0.0;
                    rec.front_face = true;
                    rec.hit = false;
                    return rec;
                }

                HitRecord hit_sphere(Sphere s, Ray r, Interval ray_t) {
                    vec3 oc = s.center - r.orig;
                    float a = dot(r.dir, r.dir);
                    float h = dot(r.dir, oc);
                    float c = dot(oc, oc) - s.radius * s.radius;
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

                    vec3 out_norm = (rec.p - s.center) / s.radius;
                    rec.front_face = get_front_face(r, out_norm);
                    rec.normal = rec.front_face ? out_norm : -out_norm;
                    rec.hit = true;

                    return rec;
                } 

                HitRecord hit_world(Ray r, Interval ray_t) {
                    HitRecord rec = new_hit_record();
                    float closest = ray_t.max;

                    for (int iter = 0; iter < sphere_count; iter++) {
                        HitRecord temp_rec = hit_sphere(spheres[iter], r, Interval(ray_t.min, closest));
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

                // Image - TODO: Move to Uniform (If we need image size outside of the camera?)
                const float image_width = 2048;
                const float image_height = 1024;

                const int samples_per_pixel = 100;
                const float pixel_samples_scale = 1.0 / float(samples_per_pixel);

                const int max_depth = 500;

                // Camera - TODO: Move to Uniform
                const float focal_length = 1.0;
                const float viewport_height = 2.0;
                const float viewport_width = viewport_height * (image_width / image_height);
                const  vec3 camera_center = vec3(0.0, 0.0, 0.0);

                // viewport vectors
                const vec3 viewport_u = vec3(viewport_width, 0, 0);
                const vec3 viewport_v = vec3(0, -viewport_height, 0);

                // pixel deltas
                const vec3 pixel_delta_u = viewport_u / image_width;
                const vec3 pixel_delta_v = viewport_v / image_height;

                // upper left pixel
                const vec3 viewport_upper_left = camera_center 
                                    - vec3(0, 0, focal_length) - viewport_u / 2 - viewport_v / 2;
                const vec3 pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

                vec3 sample_square() {
                    return vec3(rand_float() - 0.5, rand_float() - 0.5, 0);
                }

                Ray get_ray(vec2 loc) {
                    vec3 offset = sample_square();
                    vec3 pixel_center = pixel00_loc
                        + ((loc.x + offset.x) * pixel_delta_u)
                        + ((loc.y + offset.y) * pixel_delta_v);
                    vec3 ray_direction = pixel_center - camera_center;
                    Ray r = Ray(camera_center, ray_direction);
                    return r;
                }

                vec3 ray_color(Ray r, int depth) {
                    float color_mult = 1.0;
                    for (int iter = 0; iter < depth; iter++) {
                        HitRecord rec = hit_world(r, Interval(0.001, FLT_MAX));
                        if (rec.hit) {
                            vec3 dir = rec.normal + rand_vec3_unit();
                            r = Ray(rec.p, dir);
                            color_mult *= 0.5;
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
                    for (int iter = 0; iter < samples_per_pixel; iter++) {
                        Ray r = get_ray(gl_GlobalInvocationID.xy);
                        pixel_color += ray_color(r, max_depth);
                    }

                    vec4 to_write = vec4(vec_to_color(pixel_color * pixel_samples_scale), 1.0);
                    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
                }
            ",
    }
}

fn initalization() {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Could note enumerate devices")
        .next()
        .expect("no devices available");

    for family in physical_device.queue_family_properties() {
        println!(
            "Found a queue family with {:?} queue(s)",
            family.queue_count
        );
    }

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .position(|queue_family_properties| {
            queue_family_properties
                .queue_flags
                .contains(QueueFlags::GRAPHICS)
        })
        .expect("couldn't find a graphical queue family") as u32;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let image_width = 2048;
    let image_height = 1024;

    let buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        (0..image_width * image_height * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [image_width, image_height, 1],
            usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .unwrap();

    let image_view = ImageView::new_default(image.clone()).unwrap();

    let shader = cs::load(device.clone()).expect("failed to create shader modeule");

    let cs = shader.entry_point("main").unwrap();
    let stage = PipelineShaderStageCreateInfo::new(cs);
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("failed to create compute pipeline");

    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));
    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts
        .get(descriptor_set_layout_index)
        .unwrap();
    let descriptor_set = DescriptorSet::new(
        descriptor_set_allocator.clone(),
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::image_view(0, image_view.clone())],
        [],
    )
    .unwrap();

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    ));

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator.clone(),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    unsafe {
        builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .dispatch([image_width / 32, image_height / 8, 1])
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                image.clone(),
                buffer.clone(),
            ))
            .unwrap();
    }

    let command_buffer = builder.build().unwrap();

    let future: sync::future::FenceSignalFuture<
        vulkano::command_buffer::CommandBufferExecFuture<sync::future::NowFuture>,
    > = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let buffer_content = buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_width, image_height, &buffer_content[..]).unwrap();

    image.save("image.png").unwrap();

    println!("Everything Succeeded!");
}
