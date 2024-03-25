use winit::event_loop;
use winit::event_loop::*;
use winit::window::*;
use winit::event::*;

use crate::rvkp::mesh::Mesh;
use crate::rvkp::vk_impl;
use crate::rvkp::vk_renderer;
use crate::rvkp::vk_renderer::Renderer;
use crate::rvkp::{init::Vk, presenter::{VkPresenter, VkView, WINDOW_RESIZED}};
use crate::application::App;
use crate::rvkp::presenter::vert;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use crate::utils::random;
use crate::mesh_gen::{VOXGEN_CH, VoxelMeshGenJob};

use tokio::spawn;

pub async fn run(event_loop: EventLoop<()>, renderer: Arc<Mutex<Renderer>>) {  
    let renderer = renderer.lock().unwrap();

    let vk_clone = renderer.vk_impl.clone();
    let vk = vk_clone.lock().unwrap();

    vk.window.set_title("@");

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
                renderer.meshes.push(Mesh::quad(vk_clone.clone()));
            },

            _ => () 
        }
    });
}
