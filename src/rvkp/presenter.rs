use std::sync::{Arc, Mutex};

use vulkano::swapchain::Surface;

use vulkano::swapchain;
use vulkano::{Validated, VulkanError};
use vulkano::swapchain::{SwapchainPresentInfo, SwapchainAcquireFuture, PresentFuture};

use vulkano::sync::{self, GpuFuture};
use vulkano::sync::future::FenceSignalFuture;

use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::AllocationCreateInfo;

use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::{
    CommandBufferExecFuture,
    PrimaryAutoCommandBuffer, 
    allocator::StandardCommandBufferAllocator
};

use vulkano::buffer::BufferContents;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents,
};
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::format::Format; 


use vulkano::pipeline::GraphicsPipeline;
use vulkano::sync::future::JoinFuture;
use vulkano::render_pass::Framebuffer;

use crate::rvkp::shader::*;
use crate::rvkp::init::Vk;

use once_cell::sync::Lazy;

pub static WINDOW_RESIZED: Lazy<Mutex<bool>> = Lazy::new(|| {Mutex::new(false)} ); 
pub static RECREATE_SWAPCHAIN: Lazy<Mutex<bool>> = Lazy::new(|| {Mutex::new(false)} ); 

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct FVertex3d {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct InstanceData {
    #[format(R32G32B32_SFLOAT)]
    pub ofs: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub fun_factor: [f32; 3],
}

use crate::utils::random;
pub fn vert(x: f32, y: f32, z: f32) -> FVertex3d {
    FVertex3d {
        position: [x, y, z],
        color: [random(0.0, 1.0), random(0.0, 1.0), random(0.0, 1.0)],
    }
}

#[derive(Clone)]
pub struct Pipeline {
    pub viewport: Arc<Viewport>,
    pub render_pass: Arc<RenderPass>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub framebuffer: Option<Arc<Framebuffer>>,
}

use crate::rvkp::mesh::Mesh;
pub struct VkView {
    pub viewport: vulkano::pipeline::graphics::viewport::Viewport,
    pub shader_mods: Vec<Arc<vulkano::shader::ShaderModule>>,
    pub meshes: Vec<Mesh>,
    pub depth_buffer: Arc<ImageView>,
    pub surface: Arc<Surface>,
    pub framebuffers : Vec<Arc<Framebuffer>>,
    pub render_pass: Arc<vulkano::render_pass::RenderPass>,

    pub pipeline: Arc<GraphicsPipeline>,
    pub layout: Arc<vulkano::pipeline::layout::PipelineLayout>,

    pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>>>,

}

pub struct VkPresenter {
    pub frames_in_flight: usize,
    pub previous_fence_i: u32, 
    pub fences: Vec<Option<Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>>>>,
}

impl VkView {
    pub fn new(vk: Arc<Mutex<Vk>>, window: Arc<winit::window::Window>) -> Self {
        let mut vk = vk.lock().unwrap();
        let surface = Surface::from_window(vk.instance.clone(), window.clone()).unwrap();
        let viewport = vulkano::pipeline::graphics::viewport::Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let meshes = vec![
            Mesh::quad(&vk),
        ];

        
        let vs = vs::load(vk.device.clone()).unwrap();
        let fs = fs::load(vk.device.clone()).unwrap();

        vk.set_swapchain(surface.clone(), &window);
        let images = vk.images.clone().unwrap();
        let render_pass = vk.get_render_pass();
        let (pipeline, layout) = vk.get_pipeline(
            vs.clone(), 
            fs.clone(), 
            render_pass.clone(), 
            viewport.clone()
        );

        let depth_buffer = ImageView::new_default(
            Image::new(
                vk.mem_allocators.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                    extent: images[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();

        let framebuffers = vk.get_framebuffers(&render_pass, depth_buffer.clone());


        let command_buffers = vk.get_command_buffers(
            &pipeline, 
            &layout,
            &framebuffers, 
            None,
        );

        *WINDOW_RESIZED.lock().unwrap() = false;
        *RECREATE_SWAPCHAIN .lock().unwrap( )= false;

        Self {
            surface,
            render_pass,
            viewport,
            meshes,
            depth_buffer,
            shader_mods: vec![vs, fs],
            framebuffers,
            pipeline,
            layout, 
            command_buffers,
        }
    }

    pub fn if_recreate_swapchain(&mut self, window: Arc<winit::window::Window>, vk: &mut Vk) {
        let size = window.inner_size();
        let is_size_zero = size.width > 0 && size.height > 0;
        if (*WINDOW_RESIZED.lock().unwrap() || *RECREATE_SWAPCHAIN.lock().unwrap()) && is_size_zero {
            *RECREATE_SWAPCHAIN.lock().unwrap() = false;
            let new_dim = window.inner_size();

            let (new_swpchain, new_imgs) = vk.swapchain.clone().unwrap()
                .recreate(vulkano::swapchain::SwapchainCreateInfo {
                    image_extent: new_dim.into(),
                    ..vk.swapchain.clone().unwrap().create_info()
                })
            .expect("failed to recreate swpchain");

            vk.swapchain = Some(new_swpchain);
            vk.images = Some(new_imgs);
            self.framebuffers = vk.get_framebuffers(&self.render_pass, self.depth_buffer.clone());

            (self.pipeline, self.layout) = vk.get_pipeline(
                    self.shader_mods[0].clone(), 
                    self.shader_mods[1].clone(), 
                    self.render_pass.clone(), 
                    self.viewport.clone()
            );

            if *WINDOW_RESIZED.lock().unwrap() {
                *WINDOW_RESIZED.lock().unwrap() = false;

                self.viewport.extent = new_dim.into();
            }
        }
    }

    pub fn update(&mut self, vk: &mut Vk) {
        self.command_buffers = vk.get_command_buffers(
            &self.pipeline.clone(),
            &self.layout,
            &self.framebuffers,
            Some(&self)
        );

    }
}

impl VkPresenter {
    pub fn new(vk: Arc<Mutex<Vk>>) -> Self {
        let vk = vk.lock().unwrap();
        let images = vk.images.clone().unwrap();
        let frames_in_flight = images.len();
        let fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let previous_fence_i = 0;

        Self {
            frames_in_flight,
            fences,
            previous_fence_i
        }
    }

    pub fn present(&mut self, vk: &mut Vk, view: &VkView) {
        let (image_i, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(vk.swapchain.clone().unwrap(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    *RECREATE_SWAPCHAIN.lock().unwrap() = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            *RECREATE_SWAPCHAIN.lock().unwrap() = true;
        }

        if let Some(image_fence) = &self.fences[image_i as usize] {
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences[self.previous_fence_i as usize].clone() {
            None => {
                let mut now = sync::now(vk.device.clone());
                now.cleanup_finished();
                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };

        let future = previous_future
            .join(acquire_future)
            .then_execute(vk.queue.clone(), view.command_buffers[image_i as usize].clone())
            .unwrap()
            .then_swapchain_present(
                vk.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(vk.swapchain.clone().unwrap(), image_i),
            )
            .then_signal_fence_and_flush();

        self.fences[image_i as usize] = match future.map_err(Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(VulkanError::OutOfDate) => {
                *RECREATE_SWAPCHAIN.lock().unwrap() = true;
                None
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                None
            }
        };
        self.previous_fence_i = image_i;
    }
}

impl Vk {
    pub fn get_render_pass(&self) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    // Set the format the same as the swapchain.
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
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap()
    }


    pub fn get_framebuffers(
        &self,
        // images: &[Arc<Image>],
        render_pass: &Arc<RenderPass>,
        depth_buffer: Arc<ImageView>,
    ) -> Vec<Arc<Framebuffer>> {
        self.images.clone().unwrap().as_slice()
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    }

    pub fn get_pipeline(
        &self,
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> (Arc<GraphicsPipeline>, Arc<PipelineLayout>) {
        let vs = vs.entry_point("main").unwrap();
        let fs = fs.entry_point("main").unwrap();

        // let vertex_input_state = FVertex3d::per_vertex()
        //     .definition(&vs.info().input_interface)
        //     .unwrap();

        let vertex_input_state = [FVertex3d::per_vertex(), InstanceData::per_instance()]
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        (GraphicsPipeline::new(
            self.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                // depth_stencil_state: Some(depth_stencil_state),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                depth_stencil_state: Some(vulkano::pipeline::graphics::depth_stencil::DepthStencilState {
                    depth: Some(vulkano::pipeline::graphics::depth_stencil::DepthState::simple()),
                    ..Default::default()
                }),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout.clone())
            },
        )
        .unwrap(), layout)
    }

    pub fn get_command_buffers(
        &mut self,
        pipeline: &Arc<GraphicsPipeline>,
        layout: &Arc<PipelineLayout>,
        framebuffers: &[Arc<Framebuffer>],
        vk_view: Option<&VkView>,
    ) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
        framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    &self.mem_allocators.command_buffer_allocator,
                    self.queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                )
                .unwrap();

                self.camera.update();

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.1, 0.11, 0.12, 1.0].into()),
                                Some(1f32.into()),
                            ],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap();
                    // .push_constants(layout.clone(), 0, push_constant)
                

                let mut builder = self.camera.send_push_constants(builder, layout);

                if let Some(vk_view) = vk_view {
                    for mesh in &vk_view.meshes {
                        mesh.draw(&mut builder);
                    }
                 };

                builder
                    .end_render_pass(Default::default())
                    .unwrap();

                builder.build().unwrap()
            })
            .collect()

    }
}
