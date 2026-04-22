use macroquad::{prelude::*};
use simulation_pc::*;


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
        draw_particles(&particles);
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