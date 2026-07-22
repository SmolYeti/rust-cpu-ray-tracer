use vulkano::VulkanLibrary;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo,
    allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
};
use vulkano::descriptor_set::{
    DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
};
use vulkano::device::{
    Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, physical::PhysicalDeviceType,
};
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

use rand;

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
        path: "src/ray_tracing.glsl",
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, vulkano::buffer::BufferContents)]
struct Material {
    mat: u32,
    index: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, vulkano::buffer::BufferContents)]
struct Sphere {
    center_rad: [f32; 4],
    mat: Material,
    _pad: [u32; 2],
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

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(_i, q)| {
                    q.queue_flags
                        .contains(QueueFlags::GRAPHICS | QueueFlags::COMPUTE)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

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

    // Image buffer
    let image_width = 512;
    let image_height = 256;

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
    // Build scene
    let mut lambertian_materials: Vec<[f32; 4]> = vec![];
    let mut metal_materials: Vec<cs::MetalMat> = vec![];
    let dielectric_materials: Vec<f32> = vec![1.5];
    let mut spheres: Vec<Sphere> = vec![];

    // Ground
    spheres.push(Sphere {
        center_rad: [0.0, -1000.0, 0.0, 1000.0],
        mat: Material {
            mat: 0,
            index: lambertian_materials.len() as u32,
        },
        _pad: [0, 0],
    });
    lambertian_materials.push([0.5, 0.5, 0.5, 0.0]);

    for a in -11..11 {
        for b in -11..11 {
            let rand_mat: f32 = rand::random();
            let sphere: [f32; 4] = [
                a as f32 + 0.9 * rand::random::<f32>(),
                0.2,
                b as f32 + 0.9 * rand::random::<f32>(),
                0.2,
            ];

            let pos_vec: [f32; 2] = [sphere[0] - 4.0, sphere[2]];
            let dist: f32 = (pos_vec[0] * pos_vec[0] + pos_vec[1] * pos_vec[1]).sqrt();
            if dist > 0.9 {
                if rand_mat < 0.8 {
                    // diffuse
                    spheres.push(Sphere {
                        center_rad: sphere,
                        mat: Material {
                            mat: 0,
                            index: lambertian_materials.len() as u32,
                        },
                        _pad: [0, 0],
                    });
                    lambertian_materials.push([
                        rand::random(),
                        rand::random(),
                        rand::random(),
                        0.0,
                    ]);
                } else if rand_mat < 0.95 {
                    let albedo = [
                        rand::random::<f32>() * 0.5 + 0.5,
                        rand::random::<f32>() * 0.5 + 0.5,
                        rand::random::<f32>() * 0.5 + 0.5,
                    ];
                    let fuzz = rand::random::<f32>() * 0.5;
                    spheres.push(Sphere {
                        center_rad: sphere,
                        mat: Material {
                            mat: 1,
                            index: metal_materials.len() as u32,
                        },
                        _pad: [0, 0],
                    });
                    metal_materials.push(cs::MetalMat { albedo, fuzz });
                } else {
                    spheres.push(Sphere {
                        center_rad: sphere,
                        mat: Material { mat: 2, index: 0 },
                        _pad: [0, 0],
                    });
                }
            }
        }
    }

    // Sphere 1:
    spheres.push(Sphere {
        center_rad: [0.0, 1.0, 0.0, 1.0],
        mat: Material { mat: 2, index: 0 },
        _pad: [0, 0],
    });

    // Sphere 2:
    spheres.push(Sphere {
        center_rad: [-4.0, 1.0, 0.0, 1.0],
        mat: Material {
            mat: 0,
            index: lambertian_materials.len() as u32,
        },
        _pad: [0, 0],
    });
    lambertian_materials.push([0.4, 0.2, 0.1, 0.0]);

    // Sphere 3:
    spheres.push(Sphere {
        center_rad: [4.0, 1.0, 0.0, 1.0],
        mat: Material {
            mat: 1,
            index: metal_materials.len() as u32,
        },
        _pad: [0, 0],
    });
    metal_materials.push(cs::MetalMat {
        albedo: [0.7, 0.6, 0.5],
        fuzz: 0.0,
    });

    // Materials
    /*let lambertian_materials: Vec<[f32; 4]> = vec![ // Note the buffer bit
        [0.8, 0.8, 0.0, 1.0],
        [0.1, 0.2, 0.5, 1.0]
    ];
    let metal_materials: Vec<cs::MetalMat> = vec![
        cs::MetalMat{albedo: [0.8, 0.8, 0.8], fuzz: 0.3},
        cs::MetalMat{albedo: [0.8, 0.6, 0.2], fuzz: 1.0}
    ];
    let dielectric_materials: Vec<f32> = vec![1.5, 1.0 / 1.5];*/

    let lambertian_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        lambertian_materials,
    )
    .unwrap();

    let metal_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        metal_materials,
    )
    .unwrap();

    let dielectric_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        dielectric_materials,
    )
    .unwrap();

    // Spheres
    /*let spheres: Vec<Sphere> = vec![
        Sphere{center_rad: [0.0, -100.5, -1.0, 100.0], mat: Material{mat: 0, index: 0}, _pad: [0, 0]},
        Sphere{center_rad: [0.0, 0.0, -1.2, 0.5], mat: Material{mat: 0, index: 1}, _pad: [0, 0]},
        Sphere{center_rad: [-1.0, 0.0, -1.0, 0.5], mat: Material{mat: 2, index: 0}, _pad: [0, 0]},
        Sphere{center_rad: [-1.0, 0.0, -1.0, 0.4], mat: Material{mat: 2, index: 1}, _pad: [0, 0]},
        Sphere{center_rad: [1.0, 0.0, -1.0, 0.5], mat: Material{mat: 1, index: 1}, _pad: [0, 0]},
    ];*/
    let sphere_count = spheres.len() as u32;

    let spheres_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        spheres,
    )
    .unwrap();

    // Uniform buffer: Quality Parameters
    let quality_buffer: Subbuffer<cs::QualityParameters> = Buffer::new_sized(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
    )
    .unwrap();

    let quality_data = cs::QualityParameters {
        samples_per_pixel: 500,
        max_depth: 50,
        sphere_count: sphere_count,
    };

    *quality_buffer.write().unwrap() = quality_data;

    // Uniform buffer: Camera Settings
    let camera_buffer: Subbuffer<cs::CameraSettings> = Buffer::new_sized(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
    )
    .unwrap();

    let camera_data = cs::CameraSettings {
        look_from: [13.0, 2.0, 3.0].into(),
        look_at: [0.0, 0.0, 0.0].into(),
        v_up: [0.0, 1.0, 0.0].into(),
        image_width: image_width as f32,
        image_height: image_height as f32,
        vfov: 20.0,
        defocus_angle: 0.6,
        focus_dist: 10.0,
    };

    *camera_buffer.write().unwrap() = camera_data;

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
        [
            WriteDescriptorSet::image_view(0, image_view.clone()),
            WriteDescriptorSet::buffer(1, lambertian_buffer),
            WriteDescriptorSet::buffer(2, metal_buffer),
            WriteDescriptorSet::buffer(3, dielectric_buffer),
            WriteDescriptorSet::buffer(4, spheres_buffer),
            WriteDescriptorSet::buffer(5, quality_buffer),
            WriteDescriptorSet::buffer(6, camera_buffer),
        ],
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
        CommandBufferUsage::SimultaneousUse,
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
            .dispatch([image_width / 32, image_height / 32, 1])
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

    let mut copy_builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator.clone(),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    copy_builder
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            image.clone(),
            buffer.clone(),
        ))
        .unwrap();

    let copy_command_buffer = copy_builder.build().unwrap();

    let future: sync::future::FenceSignalFuture<
        vulkano::command_buffer::CommandBufferExecFuture<sync::future::NowFuture>,
    > = sync::now(device.clone())
        .then_execute(queue.clone(), copy_command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let buffer_content = buffer.read().unwrap();
    let image =
        ImageBuffer::<Rgba<u8>, _>::from_raw(image_width, image_height, &buffer_content[..])
            .unwrap();

    image.save("image.png").unwrap();

    println!("Everything Succeeded!");
}
