use winit::event_loop::*;
use winit::window::*;
use winit::event::*;

use crate::rvkp::{init::Vk, presenter::{VkPresenter, VkView, WINDOW_RESIZED}};
use crate::application::App;
use crate::rvkp::presenter::vert;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use crate::utils::random;
use crate::mesh_gen::{VOXGEN_CH, VoxelMeshGenJob};

use tokio::spawn;

pub async fn run() {  
    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    window.set_title("@");

    let vk = Arc::new(Mutex::new(Vk::new(&event_loop)));
    let mut view = Arc::new(Mutex::new(VkView::new(vk.clone(), window.clone())));
    let mut presenter =  VkPresenter::new(vk.clone());
    let _app = App::new();

    window.set_cursor_visible(false);

    // one of them gotta work
    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
    let _ = window.set_cursor_grab(CursorGrabMode::Confined);

    window.set_cursor_position(winit::dpi::PhysicalPosition::new(200.0, 200.)).unwrap();

    let mut frame_id = 0;
    let mut mouse_pos: (f64, f64) = (0.0, 0.0);

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
                *WINDOW_RESIZED.lock().unwrap() = true;
            },

            Event::WindowEvent {
                event,
                ..
            } => {
                vk.clone().lock().unwrap().camera.input(&window, &event);
            },
            
            Event::DeviceEvent {event: winit::event::DeviceEvent::MouseMotion { delta },..} => {
                mouse_pos = (mouse_pos.0 + delta.0, mouse_pos.1 + delta.1);
                vk.clone().lock().unwrap()
                    .camera.mouse_callback(-mouse_pos.0 as f32, mouse_pos.1 as f32);
            }

            Event::MainEventsCleared => {
                // let now = std::time::Instant::now();

                // window.set_cursor_position(winit::dpi::PhysicalPosition::new(200.0, 200.)).unwrap();
                let mut vk_guard = vk.lock().unwrap();
                let mut view_guard = view.lock().unwrap();

                view_guard.if_recreate_swapchain(window.clone(), &mut vk_guard);
                view_guard.update(&mut vk_guard);

                presenter.present(&mut vk_guard, &view_guard);

                frame_id += 1;

                let view_clone = view.clone();
                let vk_clone = vk.clone();
                if frame_id % 500 == 0 {
                    spawn(async {
                        VOXGEN_CH.send(VoxelMeshGenJob::chunk(), view_clone, vk_clone).await;
                    });
                }

                // dbg!(now.elapsed());
            },

            _ => () 
        }
    });
}
