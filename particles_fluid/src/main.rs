use macroquad::{miniquad::window::get_window_position, prelude::*};

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pos: Vec2,
    prev_pos: Vec2,
    vel: Vec2,
}
impl Particle {
    pub fn new(pos:Vec2, vel: Vec2 ) -> Self {
        Self {
            pos, vel, prev_pos: pos
        }
    }
}

#[macroquad::main("Particles fluid simulation")]
async fn main() {
    let mut dt;
    // macroquad::window::request_new_screen_size(particles.len() as f32*3., particles[0].len() as f32*3.);
    let w = 1224.;
    let h = 800.;
    macroquad::window::request_new_screen_size(w, h);
    next_frame().await;
    next_frame().await;

    let mut particles = Vec::with_capacity(2048);
    for _ in 0..particles.capacity() {
        let x = macroquad::rand::rand() as f32/u32::MAX as f32*w;
        let y= macroquad::rand::rand() as f32/u32::MAX as f32*h;
        let vx = macroquad::rand::rand() as f32/u32::MAX as f32*2. -1.;
        let vy= macroquad::rand::rand() as f32/u32::MAX as f32*2. -1.;
        particles.push(Particle::new(vec2(x, y), vec2(vx,vy)))
    }

    let mut screen_pos = get_window_position();

    // let restDensity = restDensity;
    // let stiffness = stiffness;
    // let nearStiffness = nearStiffness;
    // let kernelRadius = kernelRadius;

    // let maxPressure = 1;
    
    // this.useSpatialHash = true;
    // this.numHashBuckets = 5000;
    // this.particleListHeads = []; // Same size as numHashBuckets, each points to first particle in bucket list
    // this.particleListNextIdx = []; // Same size as particles list, each points to next particle in bucket list

    // this.material = new Material("water", 2, 0.5, 0.5, 40);
    let g = 1.5;


    loop {
        dt = get_frame_time()*10.;
        // dbg!(dt);
        clear_background(BLACK);


        for iterations in 0..4 {
            let new_screen_pos = get_window_position();
    
            // Update
            for p in &mut particles {
                p.vel.y += g * dt;
                p.pos.x += screen_pos.0 as f32-new_screen_pos.0 as f32;
                p.pos.y += screen_pos.1 as f32-new_screen_pos.1 as f32;
            }
            screen_pos = new_screen_pos;

            // Apply viscosity
    
    
            for p in &mut particles {
                p.prev_pos = p.pos;
                p.pos += p.vel * dt;
    
            }
            // Adjust springs
            
            // applySpringDisplacements
    
            //------------------- doubleDensityRelaxation
            let kernel_radius = 40.; // h
        
            let rest_density = 2.;
            let stiffness = 1.0;
            let near_stiffness = 0.5;
        
            // Neighbor cache
            // Indices, unit x, y, closeness
            let mut neighbors: Vec<(usize, Vec2, f32)> = vec![];
            
            let l = particles.len();
            for i in 0..l {
                let p0 = particles[i];
                let mut density = 0.;
                let mut near_density = 0.;
    
                let mut n_neighbors = 0;
                neighbors.clear();
        
                // Compute density and near-density
                for j in 0..l {
                    if i == j {continue}
                    let p1 = particles[j];
                    let diff = p1.pos - p0.pos;
        
                    if diff.x > kernel_radius || diff.x < -kernel_radius {
                        continue;
                    }
                    if diff.y > kernel_radius || diff.y < -kernel_radius {
                        continue;
                    }
        
                    if diff.length_squared() < kernel_radius*kernel_radius {
                        let r = diff.length();
                        let q = r / kernel_radius as f32;
                        let closeness = 1. - q;
                        let closeness_sq = closeness * closeness;
            
                        density += closeness * closeness;
                        near_density += closeness * closeness_sq;
            
                        neighbors.push((j, diff/r, closeness));
                        n_neighbors += 1;
                    }
                }
            
                // Add wall density
                let closest_x = p0.pos.x.min(screen_width() - p0.pos.x);
                let closest_y = p0.pos.y.min(screen_height() - p0.pos.y);
        
                if closest_x < kernel_radius {
                    let q = closest_x / kernel_radius;
                    let closeness = 1. - q;
                    let closeness_sq = closeness * closeness;
            
                    density += closeness * closeness * 1.;
                    near_density += closeness * closeness_sq * 1.;
                }
        
                if closest_y < kernel_radius {
                    let q = closest_y / kernel_radius;
                    let closeness = 1. - q;
                    let closeness_sq = closeness * closeness;
            
                    density += closeness * closeness * 1.;
                    near_density += closeness * closeness_sq * 1.;
                }
        
                // Compute pressure and near-pressure
                let pressure = stiffness * (density - rest_density);
                let near_pressure = near_stiffness * near_density;
        
                let mut disp = Vec2::ZERO;
        
                for j in 0..n_neighbors {
                    let mut p1 = particles[neighbors[j].0];
            
                    let closeness = neighbors[j].2;
                    let d = dt * dt * (pressure * closeness + near_pressure * closeness * closeness) / 2.;
                    let d = d * neighbors[j].1;
                    p1.pos += d;
                    disp -= d;
                }
        
                particles[i].pos.x += disp.x;
                particles[i].pos.y += disp.y;
            }
            
            // resolveCollisions
            let boundary_mul = 1.5 * dt; // 1 is no bounce, 2 is full bounce
            let boundary_min_x = 5.;
            let boundary_max_x = screen_width() - 5.;
            let boundary_min_y = 5.;
            let boundary_max_y = screen_height() - 5.;
    
            for p in &mut particles {
                if p.pos.x < boundary_min_x {
                    p.pos.x += boundary_mul * (boundary_min_x - p.pos.x);
                } else if p.pos.x > boundary_max_x {
                    p.pos.x += boundary_mul * (boundary_max_x - p.pos.x);
                }
    
                if p.pos.y < boundary_min_y {
                    p.pos.y += boundary_mul * (boundary_min_y - p.pos.y);
                } else if p.pos.y > boundary_max_y {
                    p.pos.y += boundary_mul * (boundary_max_y - p.pos.y);
                }
            }
    
            for p in &mut particles {
                p.vel = (p.pos-p.prev_pos) / dt;
                // p.vel *= 0.99;
            }
        }

        // Draw
        for p in particles.iter() {
            draw_circle(p.pos.x, p.pos.y, 5., RED);
        }
        
        draw_fps();
        next_frame().await
    }
}

