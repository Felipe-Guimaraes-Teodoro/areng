use winit::event_loop::*;
use winit::window::*;
use winit::event::*;

use crate::rvkp::{init::Vk, presenter::{VkPresenter, VkView, WINDOW_RESIZED}};
use crate::application::App;
use crate::rvkp::presenter::vert;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use crate::utils::random;

pub fn run() {  
    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    window.set_title("@");

    let mut vk = Vk::new(&event_loop);
    let mut view = VkView::new(&mut vk, window.clone());
    let mut presenter =  VkPresenter::new(&mut vk);
    let app = App::new();

    window.set_cursor_visible(false);
    window.set_cursor_position(winit::dpi::PhysicalPosition::new(200.0, 200.)).unwrap();

    /*
     *  struct BObj {
     *      color,
     *      transform,
     *      buffers,
     *      push_consts,
     *      ... idk 
     *  }
     */
    // view.push_b_objs(
    //     vk.vertex_buffer(vec![
    //         vert(0.1, 0.1, 0.0),
    //         vert(-0.1, 0.0, 0.0),
    //         vert(0.1, -0.1, 0.0),
    //     ]),
    //     vk.index_buffer(vec![
    //         0, 1, 2
    //     ]),
    // );

    let mut frame_id = 0.0;
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
                vk.camera.input(&window, &event);
            },
            
            Event::DeviceEvent {event: winit::event::DeviceEvent::MouseMotion { delta },..} => {
                mouse_pos = (mouse_pos.0 + delta.0, mouse_pos.1 + delta.1);
                vk.camera.mouse_callback(-mouse_pos.0 as f32, mouse_pos.1 as f32);
            }

            Event::MainEventsCleared => {
                // let now = std::time::Instant::now();

                window.set_cursor_position(winit::dpi::PhysicalPosition::new(200.0, 200.)).unwrap();
                view.if_recreate_swapchain(window.clone(), &mut vk);
                view.update(&mut vk);

                presenter.present(&mut vk, &view);

                frame_id += 1.0;

                // dbg!(now.elapsed());
            },

            _ => () 
        }
    });
}
