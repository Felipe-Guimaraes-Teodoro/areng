use std::sync::{Arc, Mutex};

use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::{EntryPoint, ShaderModule}, swapchain::SwapchainCreateInfo};
use winit::{event::{Event, WindowEvent}, event_loop::EventLoop};

use crate::utils::random;

use super::{mesh::Mesh, shader, vk_impl::VkImpl};

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
}

impl Renderer {
    pub async fn new(vk_impl: Arc<Mutex<VkImpl>>) -> Arc<Mutex<Self>> {
        let mut vk_clone = vk_impl.clone();
        let mut vk = vk_clone.lock().unwrap();
        let shaders = vec![
            shader::vs::load(vk.device.clone()).unwrap(),
            shader::fs::load(vk.device.clone()).unwrap(),
        ];

        Arc::new(Mutex::new(Self {
            vk_impl,
            meshes: vec![],
            shaders,
        }))
    }

    pub fn update(&mut self) {
        let mut vk = self.vk_impl.lock().unwrap();

        vk.if_recreate_swapchain();

        let layout = &pipeline.layout().set_layouts()[0];
        let set = DescriptorSet::new(
            descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
            [],
        )
        .unwrap();

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            recreate_swapchain = true;
        }

        let mut builder = RecordingCommandBuffer::new(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 1.0, 1.0].into()),
                        Some(1f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(
                        framebuffers[image_index as usize].clone(),
                    )
                },
                Default::default(),
            )
            .unwrap()
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                set,
            )
            .unwrap()
            .bind_vertex_buffers(0, (vertex_buffer.clone(), normals_buffer.clone()))
            .unwrap()
            .bind_index_buffer(index_buffer.clone())
            .unwrap();

        unsafe {
            builder
                .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap();
        }

        builder.end_render_pass(Default::default()).unwrap();

        let command_buffer = builder.end().unwrap();
        let future = previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
        }
        //dbg!(self.vk_impl.lock().unwrap().pipeline.clone().unwrap().flags());

        //VkImpl::window_size_dependent_setup(self.vk_impl.clone());
    }
}