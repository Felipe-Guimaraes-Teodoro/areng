mod event_loop;
mod rvkp;
mod application;
mod ui;
mod utils;
mod mesh_gen;

use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use rvkp::{init::Vk, vk_impl, vk_renderer::Renderer};
use winit::event_loop::EventLoop;

/*
 *  todo: implement the new rvkp
 */

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() {
    // mesh_gen::init().await;
    //event_loop::run().await;

    let event_loop = EventLoop::new();

    let vk = vk_impl::VkImpl::new(&event_loop).await;
    println!("vk initialized");
    let renderer = Renderer::new(vk.clone()).await;
    println!("main renderer initialized");
    vk.lock().unwrap().ignition(renderer.clone());
    println!("ignited");
    event_loop::run(event_loop, renderer.clone()).await; // now we're talking
}
