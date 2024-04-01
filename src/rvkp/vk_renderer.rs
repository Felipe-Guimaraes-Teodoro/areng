use std::sync::{Arc, Mutex};

use vulkano::{buffer::BufferContents, command_buffer::{AutoCommandBufferBuilder, RenderPassBeginInfo}, descriptor_set::WriteDescriptorSet, pipeline::{graphics::vertex_input::Vertex, Pipeline, PipelineBindPoint}, shader::{EntryPoint, ShaderModule}, swapchain::{acquire_next_image, SwapchainCreateInfo}, Validated};
use winit::{event::{Event, WindowEvent}, event_loop::EventLoop};

use crate::utils::random;

use super::{camera::Camera, mesh::Mesh, shader, vk_impl::{VkImpl, VkPresenter}};

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct RVertex3d {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
}

impl RVertex3d {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        RVertex3d {
            position: [x, y, z],
        }
    }
}

pub struct Renderer {
    pub vk_impl: Arc<Mutex<VkImpl>>,
    
    pub meshes: Vec<Mesh>,
    pub shaders: Vec<Arc<ShaderModule>>,

    pub presenter: VkPresenter,

    pub camera: Camera,
}

impl Renderer {
    pub async fn new(vk_impl: Arc<Mutex<VkImpl>>) -> Arc<Mutex<Self>> {
        let mut vk_clone = vk_impl.clone();
        let mut vk = vk_clone.lock().unwrap();
        let shaders = vec![
            shader::vs::load(vk.device.clone()).unwrap(),
            shader::fs::load(vk.device.clone()).unwrap(),
        ];
        let presenter = VkPresenter::new(&vk);
        let camera = Camera::new();

        Arc::new(Mutex::new(Self {
            vk_impl,
            meshes: vec![],
            shaders,
            presenter,
            camera,
        }))
    }
    
    pub fn update(&mut self) {
        let mut vk = self.vk_impl.lock().unwrap();

        self.presenter.if_recreate_swapchain(&mut vk);

        let mut builder = AutoCommandBufferBuilder::primary(
            &vk.allocators.clone().unwrap().command_buffer,
            vk.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let mesh_zero = || {
            self.meshes[0].clone()
        };

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 1.0, 1.0].into()),
                        Some(1f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(vk.framebuffers[0].clone())
                },
                Default::default(),
            )
            .unwrap()
            .bind_pipeline_graphics(vk.pipeline.clone().unwrap().clone())
            .unwrap();

        let mut builder = self.camera.send_push_constants(builder, vk.pipeline.clone().unwrap().layout());

        builder
            .bind_vertex_buffers(0, mesh_zero().vert_buf.unwrap().clone())
            .unwrap()
            .bind_index_buffer(mesh_zero().ind_buf.unwrap().clone())
            .unwrap()
            .draw_indexed(mesh_zero().ind_buf.unwrap().len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass(Default::default())
            .unwrap();

        let command_buffer = builder.build().unwrap();

        self.presenter.present(&vk, command_buffer);
    }
}