use std::sync::{Arc, Mutex};

use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::{EntryPoint, ShaderModule}};
use winit::{event::{Event, WindowEvent}, event_loop::EventLoop};

use super::{mesh::Mesh, shader, vk_impl::VkImpl};

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct RVertex3d {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

#[derive(Debug)]
pub struct Renderer {
    pub vk_impl: Arc<Mutex<VkImpl>>,
    
    //pub meshes: Vec<Mesh>,
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
            //meshes: vec![],
            shaders,
        }))
    }
    
    pub fn run(&mut self, event_loop: EventLoop<()>) {
        let vk_clone = self.vk_impl.clone();
        let vk = vk_clone.lock().unwrap();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { 
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                },
                
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                },
    
                Event::WindowEvent {
                    event,
                    ..
                } => {
                },
                
                Event::DeviceEvent {event: winit::event::DeviceEvent::MouseMotion { delta },..} => {
                }
    
                Event::MainEventsCleared => {

                },
    
                _ => () 
            }
        });
    }
}