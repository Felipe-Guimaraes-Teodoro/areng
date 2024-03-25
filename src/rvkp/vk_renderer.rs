use std::sync::{Arc, Mutex};

use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::{EntryPoint, ShaderModule}};
use winit::{event::{Event, WindowEvent}, event_loop::EventLoop};

use crate::utils::random;

use super::{mesh::Mesh, shader, vk_impl::VkImpl};

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct RVertex3d {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

impl RVertex3d {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        RVertex3d {
            position: [x, y, z],
            color: [random(0.0, 1.0), random(0.0, 1.0), random(0.0, 1.0)]
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
        let vk_clone = vk_impl.clone();
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
        let vk = self.vk_impl.lock().unwrap();

        //VkImpl::window_size_dependent_setup(self.vk_impl.clone());
    }
}