
use std::cell::OnceCell;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 100;
const HEIGHT: usize = 100;
const GENERATION_INFINITY: f64 = 16.;

static mut BUFFER: OnceCell<Vec<u32>> = OnceCell::new();

fn main() {
    #[allow(static_mut_refs)]
    unsafe { BUFFER.set(vec![0u32; WIDTH * HEIGHT]).unwrap() };

    let mut window = Window::new(
        "Fractal example - press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create the window");

    window.set_target_fps(60);
    window.set_background_color(0, 0, 20);

    let mut prev_mouse_pos = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();

    let mut cam_pos = Complex::with_val(128, (-0.5219751549958379, -0.6623533901957027));
    let mut zoom = Float::with_val(128, 1.0);

    let mut fract_depth_manual = 0.;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let FRACTAL_DEPTH = 32.max((zoom.to_f64().log2().abs() * 5.0) as usize) + fract_depth_manual as usize;
        let FLOAT_PRECISION: u32 = FRACTAL_DEPTH as u32;

        // cache camera/zoom as f64 for inner loop
        let cam_x_f64 = cam_pos.real().to_f64();
        let cam_y_f64 = cam_pos.imag().to_f64();
        let cam_x = cam_pos.real().clone();
        let cam_y = cam_pos.imag().clone();
        let zoom_defer = zoom.clone();
        let zoom_f64 = zoom.to_f64();
        let win_size = window.get_size();
        
        #[allow(static_mut_refs)]
        let buffer = unsafe { BUFFER.get_mut().unwrap() };
        let it = buffer.par_iter_mut();
        std::thread::Builder::new().name("fractal_render".into()).spawn(move || {
            it.enumerate().for_each(|(i, pixel)| {
                let px = (((i % WIDTH) as f64) / WIDTH as f64 * 2.0)-1.0;
                let py = (((i / WIDTH) as f64) / WIDTH as f64 * 2.0)-1.0;

                // compute world coordinates for this pixel (centered + aspect-correct)
                // let (cx, cy) = pixel_to_world(px, py, win_size.0, win_size.1, cam_x_f64, cam_y_f64, zoom_f64);

                // *pixel = (((px*500.0) as u32) << 16) | (((py*500.0) as u32) << 8);
                let (cx, cy) = (Float::with_val(FLOAT_PRECISION, px)*&zoom_defer + &cam_x, Float::with_val(FLOAT_PRECISION, py)*&zoom_defer + &cam_y);

                let a = Complex::with_val(FLOAT_PRECISION, (cx, cy));
                let prec = Float::with_val(FLOAT_PRECISION, 1e-15);
                let iter = mandel_converges(a, prec, FRACTAL_DEPTH, FLOAT_PRECISION);

                *pixel = if iter < 0 {
                    0 // Black for non-convergent points
                } else {
                    colormap(iter as usize, 4)
                };
            });
        }).unwrap().join().unwrap();

        // handle zoom (mouse wheel)
        let scroll_y = window.get_scroll_wheel().unwrap_or_default().1;
        zoom *= if scroll_y < 0.0 {
            Float::with_val(FLOAT_PRECISION, 0.6)
        } else if scroll_y > 0.0 {
            Float::with_val(FLOAT_PRECISION, 1.4)
        } else {
            Float::with_val(FLOAT_PRECISION, 1.0)
        };

        let new_mouse_pos = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
        let prev_mouse = prev_mouse_pos; // old value
        let dpos = (prev_mouse.0 - new_mouse_pos.0, prev_mouse.1 - new_mouse_pos.1);

        // If dragging, convert previous and new mouse positions to world and shift camera
        if window.get_mouse_down(minifb::MouseButton::Left) {
            *cam_pos.mut_real() += Float::with_val(
                FLOAT_PRECISION,
                dpos.0 as f64 / win_size.0 as f64,
            ) * (5. * zoom.to_f64());
            *cam_pos.mut_imag() += Float::with_val(
                FLOAT_PRECISION,
                dpos.1 as f64 / win_size.1 as f64,
            ) * (5. * zoom.to_f64());
        }
        prev_mouse_pos = new_mouse_pos;

        if window.is_key_down(Key::Up) {
            fract_depth_manual += 1.;
        }
        if window.is_key_down(Key::Down) {
            fract_depth_manual -= 1.;
            print!("Pos: ({}, {}), Zoom: {}, Fractal depth: {}\r", cam_pos.real().to_f64(), cam_pos.imag().to_f64(), zoom.to_f64(), FRACTAL_DEPTH);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }



        #[allow(static_mut_refs)]
        window.update_with_buffer(&unsafe { BUFFER.get().unwrap() }, WIDTH, HEIGHT).unwrap();
    }
}

fn map(val: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
    start2 + (stop2 - start2) * ((val - start1) / (stop1 - start1))
}


use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use rug::Float;

// #[derive(Clone)]
// struct Vec2f {
//     x: Float,
//     y: Float,
// }

// impl Vec2f {
//     fn new(x: f64, y: f64) -> Self {
//         Self { x: Float::with_val(FLOAT_PRECISION, x), y: Float::with_val(FLOAT_PRECISION, y) }
//     }
//     fn add(&self, o: &Self) -> Self {
//         Self { x: Float::with_val(FLOAT_PRECISION, &self.x + &o.x), y: Float::with_val(FLOAT_PRECISION, &self.y + &o.y) }
//     }
// }
fn hsv2rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let c = v * s;
    let h1 = h * 6.0;
    let x = c * (1.0 - (h1 / 2.0).fract() * 2.0 - 1.0).abs();
    let m = v - c;
    
    let rgb = match h1 as i32 {
        h if h < 1 => (c, x, 0.0),
        h if h < 2 => (x, c, 0.0),
        h if h < 3 => (0.0, c, x),
        h if h < 4 => (0.0, x, c),
        h if h < 5 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    (rgb.0 + m, rgb.1 + m, rgb.2 + m)
}

fn colormap(val: usize, max_iter: usize) -> u32 {
    if val == max_iter {
        return 0; // Black for points in set
    }
    
    // Convert iteration count to smooth color
    let t = val as f64 / max_iter as f64;
    
    // Create different color bands
    let h = (0.5 + t * 0.5).fract(); // Hue cycles through colors
    let s = 0.8 + 0.2 * (t * 6.28318).sin(); // Saturation oscillation
    let v = 0.7 + 0.3 * (t * 4.28318).cos(); // Value/brightness oscillation
    
    let (r, g, b) = hsv2rgb(h, s, v);
    
    // Convert to u32 RGB
    let r = (r * 255.0) as u32;
    let g = (g * 255.0) as u32;
    let b = (b * 255.0) as u32;
    
    (r << 16) | (g << 8) | b
}

use rug::Complex;

fn heron_converges(a: Complex, prec: Float, max_steps: usize, FLOAT_PRECISION: u32) -> i32 {
    // let prec_squared = prec.clone() * prec;
    
    // Initial guess using magnitude as real approximation
    let mut r = a.clone().abs().real().clone();
    if r < 1.0 {
        r = Float::with_val(FLOAT_PRECISION, 1.0);
    }
    
    let mut z = Complex::with_val(FLOAT_PRECISION, (r.clone(), 0.0));
    
    for i in 0..max_steps {
        // Newton (Heron) update: z = (z + a/z) / 2
        let a_over_z = Complex::with_val(FLOAT_PRECISION, &a / &z);
        z = (z + a_over_z) / 2.0;
        
        // Residual = z*z - a
        let resid = Complex::with_val(FLOAT_PRECISION, Complex::with_val(128, &z * &z) - &a);
        
        // Convergence test: |resid|^2 / r < prec
        if &(resid.norm().real() / r.clone()) < &prec {
            return i as i32;
        }
    }
    
    -1 // Not converged
}



fn mandel_converges(a: Complex, prec: Float, max_steps: usize, FLOAT_PRECISION: u32) -> i32 {
    // Mandelbrot set calculation with arbitrary precision using Vec2f
    let mut zx = Float::with_val(FLOAT_PRECISION, 0);
    let mut zy = Float::with_val(FLOAT_PRECISION, 0);
    let mut iteration = 0;
    while iteration < max_steps {
        let zx2 = Float::with_val(FLOAT_PRECISION, &zx * &zx);
        let zy2 = Float::with_val(FLOAT_PRECISION, &zy * &zy);

        if Float::with_val(FLOAT_PRECISION, &zx2 + &zy2) > Float::with_val(FLOAT_PRECISION, GENERATION_INFINITY) {
            break;
        }

        let new_zx = Float::with_val(FLOAT_PRECISION, &zx2 - &zy2) + Float::with_val(FLOAT_PRECISION, a.real());
        let new_zy = Float::with_val(FLOAT_PRECISION, 2) * &zx * &zy + Float::with_val(FLOAT_PRECISION, a.imag());

        zx = new_zx;
        zy = new_zy;

        iteration += 1;
    }
    iteration as i32
}

fn pixel_to_world(px: f64, py: f64, win_w: usize, win_h: usize, cam_x: f64, cam_y: f64, zoom: f64) -> (f64, f64) {
    // Convert pixel coordinates to UV space [-0.5, 0.5]
    let uv_x = px / win_w as f64 - 0.5;
    let uv_y = py / win_h as f64 - 0.5;
    
    // Apply zoom and camera position (matching WGSL shader)
    let world_x = uv_x * zoom + cam_x;
    let world_y = -uv_y * zoom + cam_y; // Flip Y to match WGSL convention
    
    (world_x, world_y)
}