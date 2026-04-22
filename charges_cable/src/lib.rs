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


pub const F: f32 = 7.0;
pub const PARTICLE_RADIUS: f32 = 5.;
pub const PARTICLE_COUNT: usize = 100;
pub const SUB_STEPS: usize = 10;

pub fn random_screen_pos() -> Vec2 {
    Vec2::new(
        macroquad::rand::gen_range(0., macroquad::window::screen_width()), 
        macroquad::rand::gen_range(0., macroquad::window::screen_height())
    )
}

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

pub fn draw_particles(particles: &[Particle]) {
    for p in particles {
        draw_circle(p.pos.x, p.pos.y, PARTICLE_RADIUS, p.ty.color());
    }
}