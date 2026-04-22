use std::ops::Add;

use bytemuck::Pod;
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton, WindowEvent},
    keyboard::KeyCode,
};

use crate::{
    buffer::{RawRect, RawRectTexture},
    get_state,
    pipelines::rect_texture::img_load_from_file,
    state::get_wgpu_state,
};

pub struct GuiManager {
    mouse_pos: Vec2f,
    buttons: Vec<Button>,
}
impl GuiManager {
    pub fn new() -> Self {
        Self {
            mouse_pos: Vec2f::ZERO,
            buttons: vec![],
        }
    }
    pub fn mouse_pos(&mut self) -> &Vec2f {
        &self.mouse_pos
    }
    pub fn mouse_move(&mut self, pos: &PhysicalPosition<f64>) {
        self.mouse_pos = vec2f(pos.x as _, pos.y as _);
        // get_state()
        //     .pipelines
        //     .rect_texture_pipeline
        //     .push_rect(RawRectTexture::new(
        //         RawRect::new(
        //             self.mouse_pos.x as f32 - 50.,
        //             self.mouse_pos.y as f32 - 50.,
        //             100.,
        //             100.,
        //         ),
        //         0,
        //     ));
    }

    pub fn mouse_click(&mut self, state: &winit::event::ElementState, button: &MouseButton) {
        match state {
            winit::event::ElementState::Pressed => {}
            winit::event::ElementState::Released => match button {
                winit::event::MouseButton::Left => {
                    for button in &self.buttons {
                        if button.rect.contains(self.mouse_pos.into()) {
                            (button.callback)()
                        }
                    }
                    self.add_button(
                        Rect::new(
                            self.mouse_pos.x as f32 - 50.,
                            self.mouse_pos.y as f32 - 50.,
                            100.,
                            100.,
                        ),
                        || dbg!(),
                    );
                }
                _ => {}
            },
        }
    }

    pub fn add_button(&mut self, rect: Rect, callback: fn()) -> &Button {
        self.buttons.push(Button { rect, callback });
        get_state().pipelines.rect_pipeline.push_rect(rect.to_raw());
        self.buttons.last().unwrap()
    }
    /// Returns true if handled interrupt
    pub(crate) fn handle_event(
        &mut self,
        event: &winit::event::WindowEvent,
        control_flow: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> bool {
        match event {
            WindowEvent::CursorMoved {
                device_id: _device_id,
                position,
            } => {
                self.mouse_move(position);
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.mouse_click(state, button);
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.state == winit::event::ElementState::Released => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(c) => match c {
                        KeyCode::KeyZ => {
                            get_state().pipelines.rect_texture_pipeline.add_text(vec2f(400., 400.), "hello");
                        }
                        KeyCode::Digit0 => {
                            get_state().pipelines.rect_texture_pipeline.push_rect(
                                RawRectTexture::new(
                                    RawRect::new(
                                        self.mouse_pos.x as f32 - 50.,
                                        self.mouse_pos.y as f32 - 50.,
                                        100.,
                                        100.,
                                    ),
                                    90,
                                ),
                            );
                        }
                        KeyCode::Digit1 => {
                            get_state().pipelines.rect_texture_pipeline.push_rect(
                                RawRectTexture::new(
                                    RawRect::new(
                                        self.mouse_pos.x as f32 - 50.,
                                        self.mouse_pos.y as f32 - 50.,
                                        100.,
                                        100.,
                                    ),
                                    1,
                                ),
                            );
                        }
                        KeyCode::Digit2 => {
                            get_state().pipelines.rect_texture_pipeline.push_rect(
                                RawRectTexture::new(
                                    RawRect::new(
                                        self.mouse_pos.x as f32 - 50.,
                                        self.mouse_pos.y as f32 - 50.,
                                        100.,
                                        100.,
                                    ),
                                    2,
                                ),
                            );
                        }
                        KeyCode::Digit3 => {
                            get_state().pipelines.rect_texture_pipeline.push_rect(
                                RawRectTexture::new(
                                    RawRect::new(
                                        self.mouse_pos.x as f32 - 50.,
                                        self.mouse_pos.y as f32 - 50.,
                                        100.,
                                        100.,
                                    ),
                                    3,
                                ),
                            );
                        }
                        KeyCode::Space => {
                            dbg!(get_state().pipelines.rect_texture_pipeline.add_texture(
                                img_load_from_file("happy-tree-64.png").unwrap().to_vec()
                            ));
                        }
                        _ => {}
                    },
                    winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                }
            }
            _ => return false,
        }
        true
    }
}
#[derive(Debug)]
pub struct Button {
    rect: Rect,
    callback: fn(),
}

pub type Rect = GenericRect<f32>;

#[derive(Debug, Copy, Clone, Default)]
pub struct GenericRect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}
impl<T> GenericRect<T> {
    pub const fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }
}
impl Rect {
    pub const EMPTY: Self = Self::new(0., 0., 0., 0.);
}
impl GenericRect<usize> {
    pub const EMPTY: Self = Self::new(0, 0, 0, 0);
}
impl<T: Add<Output = T> + Copy> GenericRect<T> {
    pub fn end(&self) -> GenericVec2<T> {
        GenericVec2::<T>::new(self.x + self.w, self.y + self.h)
    }
}
impl<T: PartialOrd + From<f32> + Add<Output = T> + Copy> GenericRect<T> {
    pub fn contains(&self, pos: Vec2f) -> bool {
        <f32 as Into<T>>::into(pos.x) >= self.x
            && <f32 as Into<T>>::into(pos.x) < self.x + self.w
            && <f32 as Into<T>>::into(pos.y) >= self.y
            && <f32 as Into<T>>::into(pos.y) < self.y + self.h
    }
}
impl Rect {
    pub(crate) fn to_raw(&self) -> RawRect {
        RawRect::new(self.x, self.y, self.w, self.h)
    }
}
pub type Vec2f = GenericVec2<f32>;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, Default, Debug)]
pub struct GenericVec2<T> {
    pub x: T,
    pub y: T,
}
impl<T> GenericVec2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
impl<T> From<PhysicalPosition<T>> for GenericVec2<T> {
    fn from(value: PhysicalPosition<T>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
impl<T: Add<Output = T>> Add for GenericVec2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Vec2f {
    pub const ZERO: Self = Self::new(0., 0.);
    const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x2];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}
unsafe impl<T: Pod> bytemuck::Pod for GenericVec2<T> {}

pub fn vec2f(x: f32, y: f32) -> Vec2f {
    Vec2f { x, y }
}
