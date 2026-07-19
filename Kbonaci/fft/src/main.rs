use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut signal = Vec::with_capacity(256);
    let freq = args.get(1).unwrap_or(&"1.0".into()).parse::<f64>().unwrap_or(1.0);
    let phase = args.get(2).unwrap_or(&"0.0".into()).parse::<f64>().unwrap_or(0.0);
    let amplitude = args.get(3).unwrap_or(&"1.0".into()).parse::<f64>().unwrap_or(1.0);
    for i in 0..signal.capacity() {
        signal.push((i as f64*freq+phase).sin()*amplitude);
    }

let mut real_planner = RealFftPlanner::<f64>::new();
let r2c = real_planner.plan_fft_forward(signal.len());
let mut spectrum = r2c.make_output_vec();
r2c.process(&mut signal, &mut spectrum).unwrap();

dbg!(spectrum);
// create an inverse FFT
// let c2r = real_planner.plan_fft_inverse(signal.len());

// // create a vector for storing the output
// let mut outdata = c2r.make_output_vec();
// assert_eq!(outdata.len(), signal.len());

// // inverse transform the spectrum back to a real-valued signal
// c2r.process(&mut spectrum, &mut outdata).unwrap();

}




use std::f32::consts::PI;

fn generate_sine(freq: f32, sample_rate: f32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|n| {
            let t = n as f32 / sample_rate;
            (2.0 * PI * freq * t).sin()
        })
        .collect()
}
