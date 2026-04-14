use std::ops::Range;

use macroquad::{prelude::*, rand::gen_range};

#[macroquad::main("Ray trace")]
async fn main() {
    let focal_length = 1.;
    let viewport_height = 2.;
    let cam_center = Vec3::ZERO;
    request_new_screen_size(800., 450.);
    next_frame().await;
    next_frame().await;

    let max_depth = 10;
    let world = World::new();

    loop {
        clear_background(RED);

        // let width = screen_width() as u16;
        // let height = screen_height() as u16;

        // let viewport_width = viewport_height * (width as f32 / height as f32);
        // let viewport_u = vec3(viewport_width, 0., 0.);
        // let viewport_v = vec3(0., -viewport_height, 0.);

        // let pixel_delta_u = viewport_u / width as f32;
        // let pixel_delta_v = viewport_v / height as f32;

        // let viewport_upper_left =
        //     cam_center - vec3(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
        // let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // let size = width as usize * height as usize;
        // let mut bytes = Vec::with_capacity(size * 4);

        // let samples_per_pixel = 10;
        // let antialiasing = if samples_per_pixel == 1 {false} else {true};
        // for x in 0..height {
        //     for y in 0..width {
        //         let mut c = Vec3::ZERO;

        //         let pixel_center =
        //             pixel00_loc + (y as f32 * pixel_delta_u) + (x as f32 * pixel_delta_v);
        //         let r = Ray {
        //             orig: pixel_center,
        //             dir: pixel_center - cam_center,
        //         };
        //         if antialiasing {
        //             for i in 0..samples_per_pixel {
        //                 let offset =
        //                     vec3(rand::gen_range(-0.5, 0.5), rand::gen_range(-0.5, 0.5), 0.);

        //                 let r = Ray {
        //                     orig: pixel_center,
        //                     dir: (pixel00_loc
        //                         + ((y as f32 + offset.x) * pixel_delta_u)
        //                         + ((x as f32 + offset.y) * pixel_delta_v))
        //                         - cam_center,
        //                 };

        //                 c += r.compute_color(&world, max_depth) / samples_per_pixel as f32;
        //             }
        //         } else {
        //             c += r.compute_color(&world, max_depth);
        //         }

        //         c = vec3(c.x.sqrt(),c.y.sqrt(),c.z.sqrt());


        //         bytes.push((c.x * 255.) as u8);
        //         bytes.push((c.y * 255.) as u8);
        //         bytes.push((c.z * 255.) as u8);
        //         bytes.push(255);
        //     }
        // }

        // let img = Image {
        //     bytes,
        //     width,
        //     height,
        // };
        // let texture = Texture2D::from_image(&img);
        // draw_texture(&texture, 0., 0., WHITE);

        let mat = load_material(ShaderSource::Glsl { vertex: DEFAULT_VERTEX_SHADER, fragment: DEFAULT_FRAGMENT_SHADER }, MaterialParams { 
            pipeline_params: PipelineParams::default(), 
            uniforms: vec![], 
            textures: vec![]
        }).unwrap();
        gl_use_material(&mat);
        draw_rectangle(0., 0., screen_width(), screen_height(), WHITE);
        gl_use_default_material();

        draw_fps();

        next_frame().await
    }
}



const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;

void main() {
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
precision lowp float;

attribute vec3 position;

void main() {
}
";
pub struct Ray {
    orig: Vec3,
    dir: Vec3,
}
impl Ray {
    pub fn new(orig: Vec3, dir: Vec3) -> Self {
        Self { orig, dir }
    }
    fn at(&self, t: f32) -> Vec3 {
        self.orig + t * self.dir
    }
    pub fn compute_color(&self, world: &World, depth: usize) -> Vec3 {
        if depth <= 0 {return Vec3::ZERO}

        if let Some(rec) = world.hit(self) {
            if let Some((ray, attenuation)) = rec.mat.scatter(self, &rec) {
                return attenuation * ray.compute_color(world, depth-1);
            } else {return Vec3::ZERO}
        }

        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        (1.0 - a) * Vec3::ONE + a * vec3(0.5, 0.7, 1.0)
    }
}

pub struct World {
    hittables: Vec<Box<dyn Hittable>>,
}
impl World {
    pub fn new() -> Self {
        let hittables: Vec<Box<dyn Hittable>> = vec![
            Box::new(Sphere {
                center: vec3(0., 0., -1.2),
                radius: 0.5,
                mat: Box::new(Lambertian::new(vec3(0.1, 0.2, 0.5))),
            }),
            Box::new(Sphere {
                center: vec3(0., -100.5, -1.),
                radius: 100.,
                mat: Box::new(Lambertian::new(vec3(0.8, 0.8, 0.))),
            }),
            Box::new(Sphere {
                center: vec3(-1., 0., -1.),
                radius: 0.5,
                mat: Box::new(Metal::new(vec3(0.8, 0.8, 0.8))),
            }),
            Box::new(Sphere {
                center: vec3(1., 0.0, -1.),
                radius: 0.5,
                mat: Box::new(Metal::new(vec3(0.8, 0.6, 0.2))),
            }),
        ];
        Self {
            hittables,
        }
    }
    pub fn hit(&self, r: &Ray) -> Option<HitRecord> {
        let mut temp_rec = None;
        let mut closest_so_far = f32::MAX;
        for obj in &self.hittables {
            if let Some(rec) = obj.hit(
                r,
                Range {
                    start: 0.01,
                    end: closest_so_far,
                },
            ) {
                closest_so_far = rec.t;
                temp_rec.replace(rec);
            }
        }
        temp_rec
    }
}



pub fn rand_unit_vec() -> Vec3 {
    loop {
        let p = vec3(gen_range(-1., 1.),gen_range(-1., 1.),gen_range(-1., 1.));
        let lensq = p.length_squared();
        if lensq <= 1. && lensq > 1e-160f32 {
            return p/lensq.sqrt();
        }
    }
}

pub fn rand_on_hemisphere(normal: Vec3) -> Vec3 {
    let v = rand_unit_vec();
    if v.dot(normal)>0. {
        v
    } else {-v}
}

pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: std::ops::Range<f32>) -> Option<HitRecord>;
}

pub struct Sphere {
    center: Vec3,
    radius: f32,
    mat: Box<dyn Material>,
}
impl<'a> Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: std::ops::Range<f32>) -> Option<HitRecord> {
        let oc = self.center - r.orig;
        let a = r.dir.length_squared();
        let h = r.dir.dot(oc);
        let c = oc.length_squared() - self.radius.powi(2);
        let discriminant = h * h - a * c;
        if discriminant < 0. {
            return None;
        }
        let sqrtd = discriminant.sqrt();
        // Find the nearest root that lies in the acceptable range.
        let root = (h - sqrtd) / a;
        if !ray_t.contains(&root) {
            let root = (h + sqrtd) / a;
            if !ray_t.contains(&root) {
                return None;
            }
        }

        let mut rec = HitRecord {
            p: r.at(root),
            normal: r.at(root) - self.center / self.radius,
            t: root,
            front_face: false,
            mat: &self.mat,
        };
        rec.set_face_normal(r, (rec.p - self.center) / self.radius);
        Some(rec)
    }
}

pub struct HitRecord<'a> {
    p: Vec3,
    normal: Vec3,
    t: f32,
    front_face: bool,
    mat: &'a Box<dyn Material>
}

impl HitRecord<'_> {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = r.dir.dot(outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Material {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)>;
}

pub struct Lambertian {
    albedo: Vec3
}
impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self {
            albedo,
        }
    }
}
impl Material for Lambertian {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)> {
        let mut dir = rec.normal + rand_unit_vec();
        if near_zero(dir) {dir = rec.normal}
        let scattered = Ray::new(rec.p, dir);
        Some((scattered, self.albedo))
    }
}

pub struct Metal {
    albedo: Vec3
}
impl Metal {
    pub fn new(albedo: Vec3) -> Self {
        Self {
            albedo,
        }
    }
}
impl Material for Metal {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)> {
        let dir = reflect(r.dir, rec.normal);
        let reflected = Ray::new(rec.p, dir);
        Some((reflected, self.albedo))
    }
}

pub fn near_zero(v: Vec3) -> bool {
    let s = 1e-8;
    (v.x.abs()<s) && (v.y.abs()<s) && (v.z.abs()<s)
}

pub fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
    v - 2. * v.dot(normal)*normal
}
