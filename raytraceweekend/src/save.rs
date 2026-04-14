use std::ops::Range;

use macroquad::prelude::*;
use wrgpgpu::*;

#[macroquad::main("Ray trace")]
async fn main() {
    let focal_length = 1.;
    let viewport_height = 2.;
    let cam_center = Vec3::ZERO;
    request_new_screen_size(800., 450.);
    next_frame().await;

    const PIXELS_PER_THREAD: usize = 10;

	let device = Device::auto_high_performance().await;
	let shader = device.create_shader::<BindGroup<StorageBufferBind<[[[u8;4]; PIXELS_PER_THREAD]; 800*600/PIXELS_PER_THREAD]>>>(ShaderArgs {
		label: "Square",
		shader: include_wgsl!("simple.wgsl"),
		entrypoint: "square",
	});




    loop {
        clear_background(RED);

        let width = screen_width() as u16;
        let height = screen_height() as u16;

        let viewport_width = viewport_height * (width as f32 / height as f32);
        let viewport_u = vec3(viewport_width, 0., 0.);
        let viewport_v = vec3(0., -viewport_height, 0.);

        let pixel_delta_u = viewport_u / width as f32;
        let pixel_delta_v = viewport_v / height as f32;

        let viewport_upper_left =
            cam_center - vec3(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let size = width as usize * height as usize;


        let buffer = StorageBufferBind::new_init(
            &device,
            [[[0, 0, 0, 1]; PIXELS_PER_THREAD]; 800*600/PIXELS_PER_THREAD],
        );
        let bind_group = device.bind(&buffer);
        device.dispatch(&shader, &bind_group, (1, 1, 1));
    
        while !device.is_complete() {
            macroquad::prelude::coroutines::wait_seconds(0.01).await;
        }
        let bytes = unsafe{std::mem::transmute(buffer.download(&device).to_vec())};

        
        
        let img = Image {
            bytes,
            width,
            height,
        };
        let texture = Texture2D::from_image(&img);
        draw_texture(&texture, 0., 0., WHITE);

        next_frame().await
    }
}

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
    pub fn compute_color(&self) -> Vec3 {
        if let Some(r) = hit(self, 0f32..f32::MAX) {
            return 0.5 * r.normal + Vec3::ONE;
        }

        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        (1.0 - a) * Vec3::ONE + a * vec3(0.5, 0.7, 1.0)
    }
}

pub fn hit(r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
    let hittables: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere {
            center: vec3(0., 0., -1.),
            radius: 0.5,
        }),
        Box::new(Sphere {
            center: vec3(0., -60., -1.),
            radius: 50.,
        }),
    ];
    let mut temp_rec = None;
    let mut closest_so_far = ray_t.end;
    for obj in hittables {
        if let Some(rec) = obj.hit(r, Range {
            start: ray_t.start,
            end: closest_so_far,
        }) {
            closest_so_far = rec.t;
            temp_rec.replace(rec);
        }
    }
    temp_rec
}

pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: std::ops::Range<f32>) -> Option<HitRecord>;
}

pub struct Sphere {
    center: Vec3,
    radius: f32,
}
impl Hittable for Sphere {
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
        if ray_t.contains(&root) {
            let root = (h + sqrtd) / a;
            if ray_t.contains(&root) {
                return None;
            }
        }

        let mut rec = HitRecord {
            p: r.at(root),
            normal: r.at(root) - self.center / self.radius,
            t: root,
            front_face: false,
        };
        rec.set_face_normal(r, (rec.p - self.center) / self.radius);
        Some(rec)
    }
}

pub struct HitRecord {
    p: Vec3,
    normal: Vec3,
    t: f32,
    front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = r.dir.dot(outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}
