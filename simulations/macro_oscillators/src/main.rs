#![allow(unused, dead_code)]
pub mod gpu;
pub mod grid;
pub mod grad_fields;

use std::f32::consts::PI;

use macroquad::prelude::*;


#[macroquad::main("Basic - Macroquad - Oscillations")]
async fn main() {
    // return gpu::main().await;
    return grid::main().await;
    // OscillationsSimulation::new().start().await;
    // grad_fields::GradFieldsSimulation::new().start().await;
}



#[derive(Clone, Debug)]
pub struct Block {
    pub h: [f32; 2],
    pub vh: [f32; 2],
    pub neighbor_indices: [Option<usize>; 2],
    pub mass: f32,
}

pub struct OscillationsSimulation {
    pub blocks: Vec<Block>,
}
impl OscillationsSimulation {
    pub fn new() -> Self {
        let mut blocks = Vec::with_capacity(screen_width() as usize);
        for i in 0..blocks.capacity() {
            let left_neighbor = if i == 0 { None } else { Some(i - 1) };
            let right_neighbor = if i == blocks.capacity() - 1 {
                None
            } else {
                Some(i + 1)
            };
            blocks.push(Block {
                h: [0.0, 0.0],
                vh: [0.0, 0.0],
                neighbor_indices: [left_neighbor, right_neighbor],
                mass: 1.0,
            });
        }
        // for i in 0..blocks.len() {
        //     blocks[i].h[0] = ((i as f32 / blocks.len() as f32) * PI * 2.0 * 2.).sin() * 100.0;
        // }
        for i in 300..350 {
            // Has  increase in a sin way
            let x = (i-300) as f32;
            blocks[i].h[0] = (5.0 * (x/(50) as f32 * PI).sin()).powi(2);
        }
        Self {
            blocks,
        }
    }
}
impl Simulation for OscillationsSimulation {
    fn base_speed(&self) -> f32 {
        80.0
    }
    fn sub_iter_count(&self) -> usize {
        2048
    }
    fn update(&mut self, dt: f32, sub_iter: usize) {
        if get_time() < 2.0 {
            // let l = self.blocks.len();
            // self.blocks[l/2].h[0] += (get_time() as f32*100.).sqrt()/2.0*dt;
        }
        if is_mouse_button_down(MouseButton::Left) {
            self.blocks[100].vh[0] += mouse_delta_position().y;
        }
        let cur = sub_iter % 2;
        let next = (sub_iter + 1) % 2;
        for i in 1..(self.blocks.len() - 1) {
            let avg_h = (self.blocks[i - 1].h[cur] + self.blocks[i + 1].h[cur]) / 2.0;
            let f = 3.0 * (avg_h - self.blocks[i].h[cur]);
            let a = f / self.blocks[i].mass;
            self.blocks[i].vh[next] = self.blocks[i].vh[cur] + a * dt;
            self.blocks[i].h[next] = self.blocks[i].h[cur] + self.blocks[i].vh[next] * dt;
        }
    }
    fn draw(&self) {
        let block_width = (screen_width()) / (self.blocks.len()) as f32;
        let block_height = 1.;
        for (i, block) in self.blocks.iter().enumerate() {
            let x = i as f32 * (block_width);
            let y = block.h[0] + screen_height() / 2.0;
            draw_line(x, y, x + block_width, y, 1.0, WHITE);
            //draw_rectangle(x, y, block_width, block_height, WHITE);
        }
        use rustfft::{FftPlanner, num_complex::Complex};

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(self.blocks.len());

        let mut buffer = Vec::with_capacity(self.blocks.len());
        for (i, block) in self.blocks.iter().enumerate() {
            buffer.push(Complex{ re: i as f32, im: block.h[0] });
        }

        fft.process(&mut buffer);
        for (i, val) in buffer.iter().enumerate() {
            draw_line(i as f32*block_width, screen_height()-20., (i) as f32 * block_width, screen_height()-20. - (val.re.abs()/20.).min(200.0), (block_width-1.0), WHITE);
        }
    }
}
            

pub trait Simulation {
    fn update(&mut self, dt: f32, update_id: usize) {}
    fn draw(&self) {}
    fn base_speed(&self) -> f32 {
        800.0
    }
    fn sub_iter_count(&self) -> usize {
        32
    }
    #[allow(async_fn_in_trait)]
    async fn start(&mut self) {
        let mut pause = false;
        let mut frame_idx = 0;
        loop {
            let dt = get_frame_time();
            if frame_idx % 10 == 0 {
                let fps = 1.0 / dt;
                print!("{fps}\r");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            clear_background(BLACK);
            if is_key_pressed(KeyCode::Space) {
                pause = !pause;
            }

            if !pause {
                let mut speed = self.base_speed();
                if is_key_down(KeyCode::Up) {
                    speed *= 3.0
                }
                if is_key_down(KeyCode::Right) {
                    speed *= 5.0
                }
                if is_key_down(KeyCode::Down) {
                    speed /= 3.0
                }
                if is_key_down(KeyCode::Left) {
                    speed /= 5.0
                }
                let sub_dt = dt / self.sub_iter_count() as f32 * speed;
                for sub_iter in 0..self.sub_iter_count() {
                    self.update(sub_dt, sub_iter);
                }
            }
            self.draw();
            frame_idx += 1;
            next_frame().await;
        }
    }
}


    // Make a smooth bump in the middle, taking at least 10 blocks:
    // let start = blocks.capacity() / 2;
    // let size = blocks.capacity() / 20;
    // // for i in start..(start+size) {
    // //     // Has  increase in a sin way
    // //     let x = (i-start) as f32;
    // //     blocks[i].h[0] = (5.0 * (x/(size) as f32 * PI).sin()).powi(3);
    // }
