use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Particle {
    pos: Vec3,
    vel: Vec3,
    color: Rgb
}
impl Particle {
    pub fn rand() -> Self {
        use fastrand::*;
        Self { 
            pos: Vec3::new(f32()*10.,f32()*10.,f32()*10.), 
            vel: Vec3::new(f32(),f32(),f32()), 
            color: Rgb::new(f32(),f32(),f32()), 
        }
    }
}

struct Model {
    particles: Vec<Particle>
}

fn model(app: &App) -> Model {
    let particles: [_; 100] = std::array::from_fn(|_| Particle::rand());
    Model {
        particles: particles.to_vec()
    }
}

fn update(app: &App, model: &mut Model, time: Update) {
    let raw = model.particles
    for p1 in model.particles.iter() {
        for p2 in model.particles.iter_mut() {

        }
    }
}

fn view(app: &App, model: &Model, frame: Frame){
    frame.clear(BLACK);
}