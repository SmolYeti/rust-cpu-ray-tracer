use vulkano::VulkanLibrary;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage, CopyImageToBufferInfo,
    allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
};
use vulkano::descriptor_set::{
    DescriptorSet, DescriptorSetWithOffsets, WriteDescriptorSet,
    allocator::StandardDescriptorSetAllocator,
};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};
use vulkano::format::{ClearColorValue, Format};
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

// Following: https://vulkano.rs/01-introduction/01-introduction.html

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

                void main() {
                    // Mandelbrot
                    /*vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
                    vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

                    vec2 z = vec2(0.0, 0.0);
                    float i;
                    for (i = 0.0; i < 1.0; i += 0.005) {
                        z = vec2(
                            z.x * z.x - z.y * z.y + c.x,
                            z.y * z.x + z.x * z.y + c.y
                        );

                        if (length(z) > 4.0) {
                            break;
                        }
                    }

                    vec4 to_write = vec4(vec3(i), 1.0);*/


                    vec4 to_write = vec4(gl_GlobalInvocationID.x / 1024.0, gl_GlobalInvocationID.y / 1024.0, 0.0, 1.0);
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

    let image_size = 1024;

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
        (0..image_size * image_size * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [image_size, image_size, 1],
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
            .dispatch([image_size / 32, image_size / 8, 1])
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
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_size, image_size, &buffer_content[..]).unwrap();

    image.save("image.png").unwrap();

    println!("Everything Succeeded!");
}
