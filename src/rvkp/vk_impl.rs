use std::{error::Error, sync::{Arc, Mutex, RwLock}};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{
        self, allocator::{CommandBufferAllocator, StandardCommandBufferAllocator}, AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferLevel, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents
    }, descriptor_set::allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator}, device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned, Queue, QueueCreateInfo, QueueFlags
    }, format::Format, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage}, instance::{Instance, InstanceCreateFlags, InstanceCreateInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState}, depth_stencil::{DepthState, DepthStencilState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo
        }, layout::PipelineDescriptorSetLayoutCreateInfo, DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo
    }, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, shader::EntryPoint, swapchain::{
        self, acquire_next_image, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo
    }, sync::{self, future::{FenceSignalFuture, JoinFuture}, GpuFuture}, Validated, VulkanError, VulkanLibrary
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::rvkp::vk_renderer::RVertex3d;

use super::{init::Vk, vk_renderer::Renderer};

#[derive(Debug)]
pub struct Allocators {
    pub memory: Arc<StandardMemoryAllocator>,
    pub descriptor_set: Arc<StandardDescriptorSetAllocator>,
    pub command_buffer: Arc<StandardCommandBufferAllocator>,
}

impl Allocators {
    fn new(device: Arc<Device>) -> Self {
        Self {
            memory: Arc::new(StandardMemoryAllocator::new_default(device.clone())),
            descriptor_set: Arc::new(StandardDescriptorSetAllocator::new(
                device.clone(),
                Default::default(),
            )),
            command_buffer: Arc::new(StandardCommandBufferAllocator::new(
                device,
                Default::default(),
            )),
        }
    }
}

pub struct VkPresenter {
    pub recreate_swapchain: bool,
    

    pub previous_frame_end: Option<Box<(dyn GpuFuture + 'static)>>,
    pub frames_in_flight: usize,
}

impl VkPresenter {
    pub fn new(vk: &VkImpl) -> Self {
        Self {
            recreate_swapchain: false,
            frames_in_flight: vk.images.len(),
            previous_frame_end: Some(sync::now(vk.device.clone()).boxed()),
        }
    }

    pub fn if_recreate_swapchain(&mut self, vk: &mut VkImpl) {
        let image_extent: [u32; 2] = vk.window.inner_size().into();
        if image_extent.contains(&0) {
            return;
        }

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = vk.swapchain.clone().unwrap()
            .recreate(SwapchainCreateInfo {
                image_extent,
                ..vk.swapchain.clone().unwrap().create_info()
            })
            .expect("failed to recreate swapchain");
    
            vk.swapchain = Some(new_swapchain);
            let (new_pipeline, new_framebuffers) = VkImpl::window_size_dependent_setup(
                vk,
            );
            vk.pipeline = Some(new_pipeline);
            vk.framebuffers = new_framebuffers;

            self.recreate_swapchain = false;
        }
    }

    pub fn present(&mut self, vk: &VkImpl, command_buffer: Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>) {
        // TODO: make this function more based off of TEAPOT.rs in vulkano github
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(vk.swapchain.clone().unwrap(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(vk.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                vk.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(vk.swapchain.clone().unwrap(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(vk.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                self.previous_frame_end = Some(sync::now(vk.device.clone()).boxed());
            }
        }

    }
}

pub struct VkImpl {
    pub window: Arc<winit::window::Window>,
    pub surface: Arc<Surface>,
    pub device: Arc<vulkano::device::Device>,
    pub queue: Arc<Queue>,

    // some fields
    pub swapchain: Option<Arc<vulkano::swapchain::Swapchain>>,
    pub images: Vec<Arc<Image>>,
    pub render_pass: Option<Arc<RenderPass>>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Option<Arc<GraphicsPipeline>>,

    pub allocators: Option<Arc<Allocators>>,

    pub renderer: Option<Arc<Mutex<Renderer>>>,
}

impl VkImpl {
    pub async fn new(event_loop: &EventLoop<()>) -> Arc<Mutex<Self>> {
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

        Arc::new(Mutex::new(Self {
            window,
            surface,
            device,
            queue,

            swapchain: None,
            images: vec![],
            render_pass: None,
            framebuffers: vec![],
            pipeline: None,

            allocators: None,

            renderer: None,
        }))
    } // new

    pub fn ignition(&mut self, renderer: Arc<Mutex<Renderer>>) {
        self.create_swapchain();
        self.create_render_pass();
        self.allocators = Some(Arc::new(Allocators::new(self.device.clone())));
        self.renderer = Some(renderer);

        let (mut pipeline, mut framebuffers) = VkImpl::window_size_dependent_setup(
            self,
        );

        self.pipeline = Some(pipeline);
        self.framebuffers = framebuffers;
    }

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
        self.images = images;
    }

    fn create_render_pass(&mut self) { 
        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: self.swapchain.clone().unwrap().image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap();

        self.render_pass = Some(render_pass);
    }

    pub fn window_size_dependent_setup(vk: &mut VkImpl)
    -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
        let device = vk.allocators.clone().unwrap().memory.device().clone();

        let vs = vk.renderer.clone().unwrap().lock().unwrap().shaders[0].clone().entry_point("main").unwrap();
        let fs = vk.renderer.clone().unwrap().lock().unwrap().shaders[1].clone().entry_point("main").unwrap();


        let extent = vk.images.clone()[0].extent();

        let depth_buffer = ImageView::new_default(
            Image::new(
                vk.allocators.clone().unwrap().memory.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                     extent: vk.images.clone()[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            ).unwrap(),
        ).unwrap();

        let framebuffers = vk.images.clone()
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    vk.render_pass.clone().unwrap(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

            let pipeline = {
                let vertex_input_state = [RVertex3d::per_vertex()]
                    .definition(&vs.info().input_interface)
                    .unwrap();
                let stages = [
                    PipelineShaderStageCreateInfo::new(vs),
                    PipelineShaderStageCreateInfo::new(fs),
                ];
                let layout = PipelineLayout::new(
                    device.clone(),
                    PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                        .into_pipeline_layout_create_info(device.clone())
                        .unwrap(),
                )
                .unwrap();
                let subpass = Subpass::from(vk.render_pass.clone().unwrap(), 0).unwrap();
        
                GraphicsPipeline::new(
                    device,
                    None,
                    GraphicsPipelineCreateInfo {
                        stages: stages.into_iter().collect(),
                        vertex_input_state: Some(vertex_input_state),
                        input_assembly_state: Some(InputAssemblyState::default()),
                        viewport_state: Some(ViewportState {
                            viewports: [Viewport {
                                offset: [0.0, 0.0],
                                extent: [extent[0] as f32, extent[1] as f32],
                                depth_range: 0.0..=1.0,
                            }]
                            .into_iter()
                            .collect(),
                            ..Default::default()
                        }),
                        rasterization_state: Some(RasterizationState::default()),
                        depth_stencil_state: Some(DepthStencilState {
                            depth: Some(DepthState::simple()),
                            ..Default::default()
                        }),
                        multisample_state: Some(MultisampleState::default()),
                        color_blend_state: Some(ColorBlendState::with_attachment_states(
                            subpass.num_color_attachments(),
                            ColorBlendAttachmentState::default(),
                        )),
                        subpass: Some(subpass.into()),
                        ..GraphicsPipelineCreateInfo::layout(layout)
                    },
                )
                .unwrap()
            };

        (pipeline, framebuffers)
    }

    pub fn vertex_buffer(&self, vertices: Vec<RVertex3d>) -> Subbuffer<[RVertex3d]> {
        Buffer::from_iter(
            self.allocators.clone().unwrap().memory.clone(), 
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default() 
            }, 
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            }, 
            vertices,
        ).unwrap()
    }

    pub fn index_buffer(&self, indices: Vec<u32>) -> Subbuffer<[u32]> {
        Buffer::from_iter(
            self.allocators.clone().unwrap().memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )
        .unwrap()
    }
}
