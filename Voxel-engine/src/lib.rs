#![feature(get_mut_unchecked)]
#![allow(
    unused_parens,
    unused_braces,
    dead_code,
    unused_import_braces,
    unused_imports,
    unused_variables
)]

pub mod buffer;
pub mod gui_manager;
pub mod pipelines;
pub mod state;
pub mod surface;
pub mod texture;
pub mod prelude;
pub mod utils;

pub use prelude::*;



pub fn run(app: impl state::App + 'static) {
    env_logger::init();
    log::set_max_level(log::LevelFilter::Debug);
    log::info!("Hi");
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    state::set_state(state::StateWrapper {
        state: None,
        app: Box::new(app),
    });
    event_loop.run_app(state::get_state_wrapper()).unwrap();
}

