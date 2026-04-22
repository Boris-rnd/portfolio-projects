use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scene {
    Prism,
    DoubleSlit,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub h: [f32; 2],
    pub vh: [f32; 2],
    // pub prev_hs: [f32; 10],
    pub mass: f32,
    // pub last_min_time: f32,
    // pub last_max_time: f32,
    // pub frequency: f32,
}

pub struct GridSimulation {
    pub row_len: usize,
    pub col_len: usize,
    pub scene: Scene,
    pub blocks: Vec<Block>,
    pub max_h: f32,
    pub img: Image,
    pub tex: Texture2D,
    pub frame_count: usize,
    pub elapsed_time: f32,
    pub chunk_rows: usize, // Number of chunks horizontally
    pub chunk_cols: usize, // Number of chunks vertically
    pub chunk_active: Vec<bool>, // Whether a chunk is currently being updated
    pub chunk_size: usize,
}

impl GridSimulation {
    pub fn new() -> Self {
        let row_len = 1000;
        let col_len = 800;
        let scene = Scene::DoubleSlit;
        let blocks = Self::create_blocks(scene, row_len, col_len);
        let img = Image::gen_image_color(row_len as u16, col_len as u16, BLACK);
        let tex = Texture2D::from_image(&img);

        Self {
            row_len,
            col_len,
            scene,
            blocks,
            max_h: 1.0,
            img,
            tex,
            frame_count: 0,
            elapsed_time: 0.0,
            chunk_rows: 50, // 1000 / 20
            chunk_cols: 40, // 800 / 20
            chunk_active: vec![true; 50 * 40], // Start all active
            chunk_size: 20,
        }
    }

    fn create_blocks(scene: Scene, row_len: usize, col_len: usize) -> Vec<Block> {
        let mut blocks = Vec::with_capacity(row_len * col_len);
        for i in 0..(row_len * col_len) {
            let x = (i % row_len) as f32;
            let y = (i / row_len) as f32;
            let mut mass = 1.0;

            match scene {
                Scene::Prism => {
                    let cx = row_len as f32 * 0.5;
                    let cy = col_len as f32 * 0.7;
                    let size = col_len as f32 * 0.4;
                    let dx = x - cx;
                    let dy = y - cy;
                    // An isosceles triangle pointing right
                    if dx > -size * 0.5 && dx < size * 0.5 && dy.abs() < (size * 0.5 - dx) {
                        mass = 1.66; // Refraction index > 1 slows down the wave
                    }
                }
                Scene::DoubleSlit => {
                    let cx = row_len as f32 * 0.5;
                    let cy = col_len as f32 * 0.5;
                    if (x - cx).abs() < 10.0 {
                        let slit_dist = col_len as f32 * 0.09;
                        let slit_radius = col_len as f32 * 0.03;
                        if (y - (cy - slit_dist)).abs() > slit_radius
                            && (y - (cy + slit_dist)).abs() > slit_radius
                        {
                            mass = f32::INFINITY; // Infinite mass = rigid wall
                        }
                    }
                }
            }
            blocks.push(Block {
                h: [0.0, 0.0],
                vh: [0.0, 0.0],
                // prev_hs: [0.0; 10],
                mass,
                // last_min_time: 0.0,
                // last_max_time: 0.0,
                // frequency: 0.0,
            });
        }
        blocks
    }

    pub fn get_block_h(&self, x: usize, y: usize, curr_idx: usize) -> f32 {
        let i = x + y * self.row_len;
        if i < self.blocks.len() {
            self.blocks[i].h[curr_idx]
        } else {
            0.0
        }
    }
}

impl crate::Simulation for GridSimulation {
    fn base_speed(&self) -> f32 {
        40.0
    }

    fn sub_iter_count(&self) -> usize {
        8
    }

    fn update(&mut self, dt: f32, sub_iter: usize) {
        if sub_iter == 0 {
            if is_key_pressed(KeyCode::R) {
                self.scene = Scene::Prism;
                self.blocks = Self::create_blocks(self.scene, self.row_len, self.col_len);
                self.frame_count = 0;
                self.elapsed_time = 0.0;
                self.img = Image::gen_image_color(self.row_len as u16, self.col_len as u16, BLACK);
                self.chunk_active.fill(true);
            }
            if is_key_pressed(KeyCode::S) {
                self.scene = Scene::DoubleSlit;
                self.blocks = Self::create_blocks(self.scene, self.row_len, self.col_len);
                self.frame_count = 0;
                self.elapsed_time = 0.0;
                self.img = Image::gen_image_color(self.row_len as u16, self.col_len as u16, BLACK);
                self.chunk_active.fill(true);
            }
            self.frame_count += 1;
            self.max_h = 1.0;
        }


        self.elapsed_time += dt;

        // Wave Spawning
        if self.elapsed_time < 5.0 {
            let cx = self.row_len as f32 * 0.2;
            let cy = self.col_len as f32 * 0.5;
            let size = 20.0;
            let min_x = (cx - 5.0 * size).max(0.0) as usize;
            let max_x = (cx + 5.0 * size).min(self.row_len as f32 - 1.0) as usize;
            let min_y = (cy - 5.0 * size).max(0.0) as usize;
            let max_y = (cy + 5.0 * size).min(self.col_len as f32 - 1.0) as usize;
            let t_sin = (self.frame_count as f32 * 0.1).sin().abs();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let i = x + y * self.row_len;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r2 = dx * dx + dy * dy;
                    let fade = (-r2 / (2.0 * size * size)).exp() / size;
                    let freq = 0.5;
                    let wave = fade * (freq * dx).cos() * t_sin * 10.0*3.;
                    self.blocks[i].h[sub_iter % 2] += wave;
                    let cx = x / self.chunk_size;
                    let cy = y / self.chunk_size;
                    self.chunk_active[cx + cy * self.chunk_rows] = true;
                }
            }
        }

        let cur = sub_iter % 2;
        let next = (sub_iter + 1) % 2;
        let row_len = self.row_len;
        let col_len = self.col_len;

        let stiffness = 6.0;
        // Frequency-preserving stability cap: ensure stiffness * dt^2 < 0.5
        let stable_stiffness = if dt * dt * stiffness > 0.5 {
            0.5 / (dt * dt + 1e-6)
            // stiffness
        } else {
            stiffness
        };

        let mut next_active = vec![false; self.chunk_active.len()];
        for cy in 0..self.chunk_cols {
            for cx in 0..self.chunk_rows {
                if !self.chunk_active[cx + cy * self.chunk_rows] {
                    continue;
                }
                let mut chunk_has_energy = false;
                let y_start = cy * self.chunk_size;
                let x_start = cx * self.chunk_size;

                for py in 0..self.chunk_size {
                    let y = y_start + py;
                    if y == 0 || y >= col_len - 1 {
                        continue;
                    }
                    for px in 0..self.chunk_size {
                        let x = x_start + px;
                        if x == 0 || x >= row_len - 1 {
                            continue;
                        }
                        let i = x + y * row_len;
                        let (left, right) = self.blocks.split_at_mut(i);
                        let (block_slice, right) = right.split_at_mut(1);
                        let block = &mut block_slice[0];

                        let avg_h = (left.last().unwrap().h[cur]
                            + right[0].h[cur]
                            + left[i - row_len].h[cur]
                            + right[row_len - 1].h[cur])
                            / 4.0;

                        let force = stable_stiffness * (avg_h - block.h[cur]);
                        // Soften extreme forces to prevent sudden jumps
                        let softening_limit = 100.0;
                        let a = if block.mass.is_infinite() {
                            0.0
                        } else {
                            if force.abs() > softening_limit {
                                (force.signum() * softening_limit) / block.mass
                            } else {
                                force / block.mass
                            }
                        };

                        // let a = force/block.mass;
                        block.vh[next] = block.vh[cur] + a * dt;
                        let mut new_h_val = block.h[cur] + block.vh[next] * dt;
                        
                        // Safety clamp: if value becomes NaN or ridiculous, reset to neighbors
                        if new_h_val.is_nan() || new_h_val.abs() > 1000.0 {
                            new_h_val = avg_h;
                            block.vh[next] = 0.0;
                        }
                        block.h[next] = new_h_val;

                        if block.h[next].abs() > 0.0001 || block.vh[next].abs() > 0.0001 {
                            chunk_has_energy = true;
                        }

                        if block.h[next] > self.max_h {
                            self.max_h = block.h[next];
                        }
                    }
                }

                if chunk_has_energy {
                    for dy in -1..=1 {
                        let ncy = cy as isize + dy;
                        if ncy >= 0 && ncy < self.chunk_cols as isize {
                            for dx in -1..=1 {
                                let ncx = cx as isize + dx;
                                if ncx >= 0 && ncx < self.chunk_rows as isize {
                                    next_active[(ncx as usize) + (ncy as usize) * self.chunk_rows] =
                                        true;
                                }
                            }
                        }
                    }
                }
            }
        }
        self.chunk_active = next_active;

        let mut img_data = self.img.get_image_data_mut();
        for (i, block) in self.blocks.iter().enumerate() {
            // let target_f = block.frequency.max(0.0).min(5.0);
            // let freq_intensity = (target_f / 5.0 * 255.0) as u8;
            let amp_intensity = ((block.h[0]).abs() * 255.0).min(255.0) as u8;

            if block.mass.is_infinite() {
                img_data[i] = [128, 128, 128, 255];
            } else if block.mass > 1.5 {
                img_data[i] = [0, 0, amp_intensity / 2, 255]; // freq_intensity
            } else {
                img_data[i] = [0, 0, amp_intensity, 255]; // freq_intensity
            }
        }

    }

    fn draw(&self) {
        self.tex.update(&self.img);
        draw_texture(&self.tex, 0., 0., WHITE);
        draw_text(
            "Space: Pause | R: Prism | S: Double Slit",
            10.0,
            20.0,
            20.0,
            WHITE,
        );
    }
}

pub async fn main() {
    use crate::Simulation;
    GridSimulation::new().start().await;
}
