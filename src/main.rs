mod event_loop;
mod rvkp;
mod application;
mod ui;
mod utils;

fn main() {
    // std::thread::spawn(|| {
    //     dbg!(utils::nth(11211024));
    // });
    // std::thread::spawn(|| {
    //     dbg!(utils::nth(112110024));
    // });
    // std::thread::spawn(|| {
    //     dbg!(utils::nth(51211024));
    // });
    event_loop::run();
    // println!("hello world")
}
