use std::sync::{Arc, Mutex};

use vulkano::shader::EntryPoint;

use super::{mesh::Mesh, shader, vk_impl::VkImpl};

pub struct Renderer {
    pub vk_impl: Arc<Mutex<VkImpl>>,
    
    pub meshes: Vec<Mesh>,
    pub shaders: Vec<EntryPoint>,

}

impl Renderer {
    pub fn new(vk_impl: Arc<Mutex<VkImpl>>) -> Self {
        let vk_clone = vk_impl.clone();
        let vk = vk_clone.lock().unwrap();
        let shaders = vec![
            shader::vs::load(vk.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap(),

            shader::fs::load(vk.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap(),
        ];

        Self {
            vk_impl,
            meshes: vec![],
            shaders,
        }
    }
}