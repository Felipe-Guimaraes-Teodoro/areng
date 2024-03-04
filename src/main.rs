mod event_loop;
mod rvkp;
mod application;
mod ui;
mod utils;

use rlua::{Error, Lua, MultiValue, RluaCompat};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

struct GlobalDataPacket {
    objects: (Vec<f32>, Vec<u32>, Vec<crate::rvkp::presenter::InstanceData>),
}

fn main() {
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

    event_loop::run();
    // std::thread::spawn(|| {
    //     dbg!(utils::nth(112110024));
    // });
    // std::thread::spawn(|| {
    //     dbg!(utils::nth(51211024));
    // });
    // event_loop::run();
    // println!("hello world")
}
