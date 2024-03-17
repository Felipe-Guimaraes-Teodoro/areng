mod event_loop;
mod rvkp;
mod application;
mod ui;
mod utils;
mod mesh_gen;

use rlua::Lua;
use rvkp::{init::Vk, vk_impl};
use winit::event_loop::EventLoop;

/*
 *  todo: rewrite rvkp 
 */

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() {
    std::thread::spawn(|| {
        let lua = Lua::new();

        loop {
            lua.globals().set(
                "hello",
                lua.create_function(|_, n: u32| {
                    println!("{:?}", utils::nth(n));
                    return Ok(());
                }).unwrap(),
            ).unwrap();

            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input); 
            if let Ok(_chunk) = lua.load(input).set_name("user").exec() {}
        }
    });

    // mesh_gen::init().await;
    //event_loop::run().await;

    let event_loop = EventLoop::new();

    vk_impl::VkImpl::init(event_loop);
}
