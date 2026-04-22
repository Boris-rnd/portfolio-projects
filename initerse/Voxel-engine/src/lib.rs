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

pub use state::{get_state,get_wgpu_state, State};
use winit::{event::Event, event_loop::EventLoop};

pub struct ExampleApp {}
impl App for ExampleApp {
    fn setup(&mut self, state: &mut State) {
        state
            .gui_manager
            .add_button(Rect::new(500., 500., 200., 75.), || {
                dbg!();
            });
    }

    fn update(&mut self) {}

    fn handle_event(&mut self, event: &Event<()>) {}
}

pub trait App {
    /// Called once at beginning of loop
    fn setup(&mut self, state: &mut State);
    /// Called every frame (after main draw and screen clear)
    fn update(&mut self);
    /// Called for ALL events
    fn handle_event(&mut self, event: &Event<()>);
}

pub fn setup_window() -> EventLoop<()> {
    pollster::block_on(async {
        env_logger::init();
        let event_loop = EventLoop::new().unwrap();
        let b = winit::window::WindowBuilder::new().with_theme(Some(winit::window::Theme::Dark));
        let window = b.build(&event_loop).unwrap();
        let state = State::new(window).await;
        unsafe { state::STATE.replace(state) };
        event_loop
    })
}

pub fn run(app: &mut dyn App) {
    let event_loop = setup_window();
    let state = state::get_state();
    app.setup(state);
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
                    app.update()
                }
                _ => {}
            };
            app.handle_event(&event);
        })
        .unwrap();
}
