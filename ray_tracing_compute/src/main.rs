use vulkano::VulkanLibrary;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
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
    mat : Material,
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

    // Image buffer
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

    // Materials
    let lambertian_materials: [[f32; 4]; _] = [ // Note the buffer bit
        [0.8, 0.8, 0.0, 1.0],
        [0.1, 0.2, 0.5, 1.0]
    ]; 
    let metal_materials: [cs::MetalMat; _] = [
        cs::MetalMat{albedo: [0.8, 0.8, 0.8], fuzz: 0.3},
        cs::MetalMat{albedo: [0.8, 0.6, 0.2], fuzz: 1.0}
    ];
    let dielectric_materials: [f32; _] = [1.5, 1.0 / 1.5];
    
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
    let spheres: [Sphere; _] = [
        Sphere{center_rad: [0.0, -100.5, -1.0, 100.0], mat: Material{mat: 0, index: 0}, _pad: [0, 0]},
        Sphere{center_rad: [0.0, 0.0, -1.2, 0.5], mat: Material{mat: 0, index: 1}, _pad: [0, 0]},
        Sphere{center_rad: [-1.0, 0.0, -1.0, 0.5], mat: Material{mat: 2, index: 0}, _pad: [0, 0]},
        Sphere{center_rad: [-1.0, 0.0, -1.0, 0.4], mat: Material{mat: 2, index: 1}, _pad: [0, 0]},
        Sphere{center_rad: [1.0, 0.0, -1.0, 0.5], mat: Material{mat: 1, index: 1}, _pad: [0, 0]},
    ];

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
        samples_per_pixel: 100,
        max_depth: 100,
        sphere_count: spheres.len() as i32,
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
        look_from: [-2.0, 2.0, 1.0].into(),
        look_at: [0.0, 0.0, -1.0].into(),
        v_up: [0.0, 1.0, 0.0].into(),
        image_width: 2048.0,
        image_height: 1024.0,
        vfov: 20.0,
        defocus_angle: 0.0,
        focus_dist: 3.4,
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
            WriteDescriptorSet::buffer(
                1,
                lambertian_buffer
            ),
            WriteDescriptorSet::buffer(
                2,
                metal_buffer
            ),
            WriteDescriptorSet::buffer(
                3,
                dielectric_buffer
            ),
            WriteDescriptorSet::buffer(
                4,
                spheres_buffer
            ),
            WriteDescriptorSet::buffer(
                5,
                quality_buffer
            ),
            WriteDescriptorSet::buffer(
                6,
                camera_buffer
            ),
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
    let image =
        ImageBuffer::<Rgba<u8>, _>::from_raw(image_width, image_height, &buffer_content[..])
            .unwrap();

    image.save("image.png").unwrap();

    println!("Everything Succeeded!");
}
