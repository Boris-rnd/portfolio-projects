pub mod stress;
pub mod rect_text;

// pub fn run_tests() {
    // if let Err(e) = env_logger::try_init() {
    //     dbg!(e);
    // }
    // let event_loop = winit::event_loop::EventLoop::new().unwrap();
    // // #[cfg(feature = "testing")]c
    // let b = winit::window::WindowBuilder::new().with_theme(Some(winit::window::Theme::Dark));
    // let window = b.build(&event_loop).unwrap();
    // let state = pollster::block_on(async {easy_wgpu::State::new(window).await});
    // unsafe { easy_wgpu::state::STATE.replace(state) };
    // let a = run::<stress::StressedApp>;
    // run::<stress::StressedApp>(event_loop);
    // run::<rect_text::RectTextApp>(event_loop);
// }



// fn run<T: easy_wgpu::App + 'static>(event_loop: winit::event_loop::EventLoop<()>) {
//     let state = easy_wgpu::get_state();
//     let mut id = 0;
//     let to_run = [
//         run::<stress::StressedApp>,
//         run::<rect_text::RectTextApp>
//     ];

//     unsafe { easy_wgpu::state::APP.replace(Box::new(T::setup(state))) };
//     event_loop
//         .run(move |event, control_flow| {
//             match event {
//                 winit::event::Event::WindowEvent { window_id: _, event: winit::event::WindowEvent::CloseRequested } => {
//                     id += 1;
//                     unsafe { easy_wgpu::state::APP.replace(Box::new(T::setup(state))) };
//                 },
//                 winit::event::Event::WindowEvent {
//                     ref event,
//                     window_id,
//                 } if window_id == state.window().id() => state.input(event, control_flow),
//                 winit::event::Event::AboutToWait => {
//                     // RedrawRequested will only trigger once unless we manually
//                     // request it.
//                     state.window().request_redraw();
//                 }
//                 _ => {}
//             };
//             state.get_app().handle_event(&event);
//         })
//         .unwrap();
// }