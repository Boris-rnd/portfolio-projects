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

pub use gui_manager::*;

use state::APP;
pub use state::{get_state, State, get_wgpu_state};
use winit::{event::Event, event_loop::{self, EventLoop}};

pub struct EmptyApp {}
impl App for EmptyApp {
    fn setup(state: &mut State) -> Self {Self {}}

    fn render(&mut self, screen: &wgpu::TextureView) {}

    fn handle_event(&mut self, event: &Event<()>) {}
}

pub trait App {
    /// Called once at beginning of loop
    fn setup(state: &mut State) -> Self where Self: Sized;
    /// Called every frame (after main draw and screen clear)
    fn render(&mut self, screen: &wgpu::TextureView);
    /// Called for ALL events
    fn handle_event(&mut self, event: &Event<()>);
}


pub fn run<T: App + 'static>() {
    if let Err(e) = env_logger::try_init() {
        dbg!(e);
    }
    let event_loop = EventLoop::new().unwrap();
    // #[cfg(feature = "testing")]c
    let b = winit::window::WindowBuilder::new().with_theme(Some(winit::window::Theme::Dark));
    let window = b.build(&event_loop).unwrap();
    let state = pollster::block_on(async {State::new(window).await});
    unsafe { state::STATE.replace(state) };
    let state = state::get_state();
    let app = T::setup(state);
    unsafe { APP.replace(Box::new(app)) };
    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => state.input(event, control_flow),
                Event::AboutToWait => {
                    // RedrawRequested will only trigger once unless we manually
                    // request it.
                    state.window().request_redraw();
                }
                _ => {}
            };
            state.get_app().handle_event(&event);
        })
        .unwrap();
}
