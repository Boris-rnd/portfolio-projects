use macroquad::{prelude::*, rand::gen_range};

pub struct GradFieldsSimulation {
    pub field: Vec<Vec<Vec2>>,
    pub particles: Vec<Particle>,
    pub field_row_len: usize,
    pub field_col_len: usize,
    pub max_len: f32,
}

impl GradFieldsSimulation {
    pub fn new() -> Self {
        let field_row_len = 200;
        let field_col_len = 200;
        let mut field = vec![vec![vec2(0.0, 0.0); field_col_len]; field_row_len];
        let mut particles = Vec::new();
        particles.push(Particle {
            pos: vec2(600.0, 500.0),
            v: vec2(0.0, -10.0),
            charge: 1.0,
        });
        particles.push(Particle {
            pos: vec2(800.0, 500.0),
            v: vec2(0.0, 10.0),
            charge: -1.0,
        });
        // for i in 0..100 {
        //     particles.push(Particle {
        //         pos: vec2(gen_range(0., screen_width()), gen_range(0., screen_height())),
        //         v: vec2(gen_range(-10., 10.), gen_range(-10., 10.)),
        //         charge: gen_range(-1f32, 1.).signum(),
        //     });

        // }
        Self {
            field,
            particles,
            field_row_len,
            field_col_len,
            max_len: 0.0,
        }
    }
}

impl crate::Simulation for GradFieldsSimulation {
    fn sub_iter_count(&self) -> usize {
        1
    }
    fn base_speed(&self) -> f32 {
        1.0
    }
    fn update(&mut self, dt: f32, update_id: usize) {
        for particle_idx in 0..self.particles.len() {
            let mut dv = vec2(0.0, 0.0);
            for particle2 in self.particles.iter() {
                let d = self.particles[particle_idx].pos - particle2.pos;
                if d.length_squared() < 50.0 {
                    continue;
                }
                let f = (self.particles[particle_idx].charge*particle2.charge*40000.0) / d.length_squared();
                dv += f * d.normalize() * dt;
            }
            self.particles[particle_idx].v += dv;
            let v = self.particles[particle_idx].v;
            self.particles[particle_idx].pos += v * dt;
            if self.particles[particle_idx].pos.x < 0.0 || self.particles[particle_idx].pos.x > screen_width() || self.particles[particle_idx].pos.y < 0.0 || self.particles[particle_idx].pos.y > screen_height() {
                self.particles[particle_idx].v *= -1f32;
            }
        }
            

        for (local_y, field_line) in self.field.iter_mut().enumerate() {
            for (local_x, field_point) in field_line.iter_mut().enumerate() {
                let screen_x = local_x as f32 / self.field_row_len as f32 * screen_width();
                let screen_y = local_y as f32 / self.field_col_len as f32 * screen_height();
                *field_point = vec2(0.0, 0.0);
                for particle in self.particles.iter() {
                    let d = particle.pos - vec2(screen_x, screen_y);
                    let f = (particle.charge*40000.0) / d.length_squared();
                    *field_point += f * d.normalize();
                }
                if field_point.length().sqrt() > self.max_len {
                    self.max_len = field_point.length().sqrt();
                }
            }
        }
    }
    fn draw(&self) {
        for (y, field_line) in self.field.iter().enumerate() {
            for (x, field_point) in field_line.iter().enumerate() {
                let screen_x = x as f32 / self.field_row_len as f32 * screen_width();
                let screen_y = y as f32 / self.field_col_len as f32 * screen_height();
                let color = if field_point.x > 0.0 { RED } else { BLUE };
                let len_corrector = (field_point.length().sqrt() / self.max_len*150.).min(15.);
                let mut field_point_x = field_point.x * len_corrector;
                let mut field_point_y = field_point.y * len_corrector;
                if field_point_x< -15. {
                    field_point_x = -15.;
                }
                if field_point_x>15. {
                    field_point_x = 15.;
                }
                if field_point_y< -15. {
                    field_point_y = -15.;
                }
                if field_point_y>15. {
                    field_point_y = 15.;
                }
                // draw_line(screen_x, screen_y, screen_x + field_point.x, screen_y + field_point.y, 5., color);
                let angle = field_point_y.atan2(field_point_x);
                // Make a rainbow color from the angle, to express phase (0deg is red and rotates to make full rainbow color)
                // Following code from: https://github.com/Inseckto/HSV-to-RGB/blob/master/HSV2RGB.c
                let hue = (angle / (2.0 * std::f32::consts::PI) + 0.5).rem_euclid(1.0);
                let i = (hue * 6.0).floor() as i32;
                let f = hue * 6.0 - i as f32;
                let p = 0.0;
                let q = 1.0 - f;
                let t = f;

                let (r, g, b) = match i % 6 {
                    0 => (1.0, t, p),
                    1 => (q, 1.0, p),
                    2 => (p, 1.0, t),
                    3 => (p, q, 1.0),
                    4 => (t, p, 1.0),
                    _ => (1.0, p, q),
                };

                let r = r*(field_point.length()*100.).clamp(0., 255.);
                let g = g*(field_point.length()*100.).clamp(0., 255.);
                let b = b*(field_point.length()*100.).clamp(0., 255.);

                let color = Color::from_rgba(r as u8, g as u8, b as u8, 255);
                draw_rectangle(screen_x, screen_y, screen_width()/self.field_row_len as f32, screen_height()/self.field_col_len as f32, color);
            }
        }
        for particle in self.particles.iter() {
            let color = if particle.charge > 0.0 { RED } else { BLUE };
            draw_circle(particle.pos.x, particle.pos.y, 5.0, color);
        }
    }
}

pub struct Particle {
    pub pos: Vec2,
    pub v: Vec2,
    pub charge: f32,
}

