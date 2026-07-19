use macroquad::prelude::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nucleon {
    Proton,
    Electron,
    Neutron,
}
impl Nucleon {
    pub fn charge(&self) -> f32 {
        match self {
            Nucleon::Proton => 1.0,
            Nucleon::Electron => -1.,
            Nucleon::Neutron => 0.,
        }
    }
    pub fn color(&self) -> Color {
        match self {
            Nucleon::Proton => RED,
            Nucleon::Electron => BLUE,
            Nucleon::Neutron => GRAY,
        }
    }
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub mass: f32,
    pub ty: Nucleon,
}
impl Particle {
    pub fn electron(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            ty: Nucleon::Electron,
            mass: 1.,
        }
    }
    pub fn proton(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            ty: Nucleon::Proton,
            mass: 1835.,
        }
    }
    pub fn neutron(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            ty: Nucleon::Neutron,
            mass: 1835.,
        }
    }
}
pub struct Rule {
    pub ty1: Nucleon,
    pub ty2: Nucleon,
    pub force: f32,
}
impl Rule {
    pub const fn new(ty1: Nucleon, ty2: Nucleon, force: f32) -> Self {
        Self {
            ty1,
            ty2,
            force,
        }
    }
}


pub fn random_screen_pos() -> Vec2 {
    Vec2::new(
        macroquad::rand::gen_range(0., macroquad::window::screen_width()), 
        macroquad::rand::gen_range(0., macroquad::window::screen_height())
    )
}


pub fn draw_particles(particles: &[Particle], particle_radius: f32) {
    for p in particles {
        draw_circle(p.pos.x, p.pos.y, particle_radius, p.ty.color());
    }
}