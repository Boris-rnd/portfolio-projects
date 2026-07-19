use macroquad::{prelude::*};
use simulation_pc::*;

pub const F: f32 = 7.0;
pub const PARTICLE_RADIUS: f32 = 5.;
pub const PARTICLE_COUNT: usize = 100;
pub const SUB_STEPS: usize = 10;


pub const DEFAULT_RULES: &[Rule] = &[
    Rule::new(Nucleon::Electron, Nucleon::Proton, -F/2.),
    Rule::new(Nucleon::Proton, Nucleon::Electron, -F/2.),
    Rule::new(Nucleon::Electron, Nucleon::Electron, F),
    Rule::new(Nucleon::Proton, Nucleon::Proton, F),
    
    Rule::new(Nucleon::Neutron, Nucleon::Proton, -F/2.),
    Rule::new(Nucleon::Proton, Nucleon::Neutron, -F/2.),
];

pub fn update_particles(particles: &mut [Particle], rules: &[Rule]) {
    let l = particles.len();
    for _j in 0..SUB_STEPS {
        for i in 0..l {
            for j in 0..l {
                if i == j { continue; }
                let p2 = &particles[j]; // Immutable borrow is fine here
                let dst = particles[i].pos.distance(p2.pos);
                let dir = (particles[i].pos - p2.pos).normalize();

                for r in rules {
                    if p2.ty == r.ty2 && particles[i].ty == r.ty1 {
                        let mut f = r.force * F /(dst.powi(2));
                        if dst < 5. {f*=-1.}
                        particles[i].vel += dir * f/(SUB_STEPS as f32);
                        break
                    }
                }
            }
        }
    }
    for p in particles {
        if p.pos.x+PARTICLE_RADIUS/2. >= screen_width() {p.pos.x = screen_width()-PARTICLE_RADIUS/2.}
        else if p.pos.x-PARTICLE_RADIUS/2. <= 0. {p.pos.x = PARTICLE_RADIUS/2.}
        if p.pos.y+PARTICLE_RADIUS/2. >= screen_height() {p.pos.y = screen_height()-PARTICLE_RADIUS/2.}
        else if p.pos.y-PARTICLE_RADIUS/2. <= 0. {p.pos.y = PARTICLE_RADIUS/2.}
        if p.pos.x > screen_width()-20. && p.ty == Nucleon::Electron {p.pos.x = 10.}
        
        
        p.pos += p.vel/p.mass;
        p.vel *= 0.99;
    }
}


#[macroquad::main("MyGame")]
async fn main() {
    #[allow(non_snake_case)]
    let mut running: bool = true;

    let mut particles = vec![];
    for _ in 0..PARTICLE_COUNT {
        particles.push(Particle::electron(random_screen_pos()));
    }

    loop {
        clear_background(BLACK);
        if running {
            update_particles(&mut particles, DEFAULT_RULES);
        }
        draw_particles(&particles, PARTICLE_RADIUS);
        if is_mouse_button_pressed(MouseButton::Left) {
            particles.push(Particle::electron(mouse_position().into()));
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            particles.push(Particle::proton(mouse_position().into()));
        }
        if is_mouse_button_pressed(MouseButton::Middle) {
            particles.push(Particle::neutron(mouse_position().into()));
        }
        if is_key_pressed(KeyCode::Space) {
            running = !running;
        }

        next_frame().await
    }
}