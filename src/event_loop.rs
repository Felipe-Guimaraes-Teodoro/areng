use winit::event_loop::*;
use winit::window::*;
use winit::event::*;

use crate::rvkp::{init::Vk, presenter::{VkPresenter, VkView, WINDOW_RESIZED}};
use crate::application::App;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

pub fn run() {  
    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    window.set_title("@");

    let mut vk = Vk::new(&event_loop);
    let mut view = VkView::new(&mut vk, window.clone());
    let mut presenter =  VkPresenter::new(&mut vk);
    let app = App::new();

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

            Event::MainEventsCleared => {
                // let then = std::time::Instant::now();

                view.if_recreate_swapchain(window.clone(), &mut vk);
                view.update(&mut vk);
                presenter.present(&mut vk, &view);

                // println!("@MAIN: MainEventsCleared cleared within {:?}", then.elapsed());
            },

            _ => () 
        }
    });
}
