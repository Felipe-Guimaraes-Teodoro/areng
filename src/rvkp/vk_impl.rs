use std::{error::Error, sync::{Arc, Mutex}};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferLevel,
        CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags
    },
    image::{view::ImageView, Image, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        self, acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo
    },
    sync::{self, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use super::init::Vk;

struct Allocators {
    memory: Arc<StandardMemoryAllocator>,
}

impl Allocators {
    fn new(device: Arc<Device>) -> Self {
        Self {
            memory: Arc::new(StandardMemoryAllocator::new_default(device))
        }
    }
}

pub struct VkImpl {
    window: Arc<winit::window::Window>,
    event_loop: Arc<winit::event_loop::EventLoop<()>>,
    surface: Arc<Surface>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<Queue>,

    // some fields
    swapchain: Option<Arc<vulkano::swapchain::Swapchain>>,
    images: Option<Vec<Arc<Image>>>,

    allocators: Option<Allocators>,
}

impl VkImpl {
    pub fn init() -> Arc<Mutex<Self>> {
        let mut this = Self::new();
        this.create_swapchain();
        this.allocators = Some(Allocators::new(this.device.clone()));

        Arc::new(Mutex::new(this))
    }

    pub fn new() -> Self {
        let event_loop = EventLoop::new();

        let library = VulkanLibrary::new().unwrap();

        let required_extensions = Surface::required_extensions(&event_loop);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| {
                p.supported_extensions().contains(&device_extensions)
            })
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            })
            .expect("no suitable physical device found");

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],

                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        Self {
            window,
            event_loop: event_loop.into(),
            surface,
            device,
            queue,

            swapchain: None,
            images: None,

            allocators: None,
        }
    } // new

    fn create_swapchain(&mut self) {
        let (swapchain, images) = {
            let surface_capabilities = self.device
                .physical_device()
                .surface_capabilities(&self.surface, Default::default())
                .unwrap();
            let image_format = self.device
                .physical_device()
                .surface_formats(&self.surface, Default::default())
                .unwrap()[0]
                .0;

            Swapchain::new(
                self.device.clone(),
                self.surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: self.window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        self.swapchain = Some(swapchain);
        self.images = Some(images);
    }
}