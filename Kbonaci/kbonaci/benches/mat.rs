#![allow(unused, dead_code, incomplete_features)]

use std::{hint::black_box, ops::{Add, Mul}};
use faer::traits::MulByRef;
use kbonaci::{mat::Matrix, *};

// pub fn criterion_benchmark(c: &mut Criterion) {
//     let input = Matrix::new([[4u64; 10]; 10]);
//     c.bench_with_input(BenchmarkId::new("Mat add", input.values.len()), &input, |b, input| b.iter(|| {
//         input+input
//     }));
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);


// pub fn plot_bench() {
//     for input_size in (1..1000usize).step_by(10) {
//         let input = Matrix::new([[4u64; input_size]]);
//         // println!("{}", input.to_string().len());
//         let start = std::time::Instant::now();
//         let output = std::hint::black_box(big_explicit_fib(input, p));
//         let duration = start.elapsed();
//         dbg!(i, duration);
//         println!(
//             "Time elapsed in expensive_function() is: {} - {:?}",
//             output.to_string(),
//             duration
//         );
//         // dbg!(output);
//     }
// }


use std::collections::{BTreeMap, HashMap};

fn main() {
    let mut kbonnaci_times = BTreeMap::new();

    // bench_and_insert(Matrix::new([[4.0f32; 1]; 1]), 1, |a| {black_box(a+a);}, &mut kbonnaci_times);
    bench_and_insert(Matrix::new([[4.0f32; 4]; 4]), 0, |a| {black_box(a*a);}, &mut kbonnaci_times);
    bench_and_insert(Matrix::new([[4.0f32; 4]; 4]), 1, |a| {black_box(a+a);}, &mut kbonnaci_times);
    // bench_and_insert(Matrix::new([[4.0f32; 100]; 100]), 100, |a| {black_box(a+a);}, &mut kbonnaci_times);
    // bench_and_insert(Matrix::new([[4.0f32; 500]; 500]), 500, |a| {black_box(a+a);}, &mut kbonnaci_times);
    // bench_and_insert(Matrix::new([[4.0f32; 100]; 100]), 100, |a| {black_box(a+a);}, &mut kbonnaci_times);

    // let fours = glam::Vec4::splat(4.);
    // let mut glam = BTreeMap::new();
    // bench_and_insert(glam::mat4(fours,fours,fours,fours), 4, |a| {black_box(a*a);}, &mut glam);
    // bench_and_insert(glam::mat4(fours,fours,fours,fours), 5, |a| {black_box(a*a);}, &mut glam);
    // bench_and_insert(glam::mat4(fours,fours,fours,fours), 6, |a| {black_box(a*a);}, &mut glam);

    // let fours = ndarray::Array::from_elem((4, 4), 4.0f32);
    // let mut ndarr = BTreeMap::new();
    // bench_and_insert(fours.clone(), 4, |a| {black_box(a*a);}, &mut ndarr);
    // bench_and_insert(fours.clone(), 5, |a| {black_box(a*a);}, &mut ndarr);
    // bench_and_insert(fours.clone(), 6, |a| {black_box(a*a);}, &mut ndarr);


    // let mut i = 1;
    // let mut j = 0;
    // while i < input_size {
    //     if i < 10_000 {
    //         i += 500;
    //     } else {
    //         i = (i as f64 * 1.3) as usize;
    //     }
    //     let input = Matrix::new();
    //     let start = std::time::Instant::now();
    //     let _ = std::hint::black_box();
    //     let elapsed = start.elapsed().as_millis();
    //     kbonnaci_times.insert(i, elapsed);
    //     j += 1;
    //     if j % 4 == 0 {
    //         print!("{i}/{max_iter}\r");
    //         std::io::Write::flush(&mut std::io::stdout()).unwrap();
    //     }
    // }
    
    let fours = faer::Mat::full(4, 4, 4.0f32);
    let mut faer = BTreeMap::new();
    bench_and_insert(fours.clone(), 0, |a| {black_box(a*a);}, &mut faer);
    bench_and_insert(fours.clone(), 1, |a| {black_box(a+a);}, &mut faer);

    let mut faer = BTreeMap::new();
    bench_and_insert(fours.clone(), 0, |a| {black_box(a*a);}, &mut faer);
    bench_and_insert(fours.clone(), 1, |a| {black_box(a+a);}, &mut faer);

    faer::Mat::mul(faer::Mat::full(4, 4, 4.0f32), faer::Mat::full(4, 4, 4.0f32));

    dict_plot!("Mat add benchmark", kbonnaci_times, faer);
}

fn bench_and_insert<T>(input: T, key: usize, f: impl Fn(&T), times: &mut BTreeMap<usize, u128>) {
    let start = std::time::Instant::now();
    for _ in 0..100_000_00 {
        std::hint::black_box(std::hint::black_box(&f)(std::hint::black_box(&input)));
    }
    let elapsed = start.elapsed().as_millis();
    times.insert(key, elapsed);
}



#[macro_export]
macro_rules! dict_plot {
    ($title:expr, $($ys:expr), +) => {{
        use simple_plot::_plotly::{Mode, Title, Plot, Scatter,Layout};
        let layout = Layout::new().title(Title::new($title));
        let mut plot = Plot::new();
        $(
            let name = stringify!($ys);
            let (xs, ys):(Vec<_>, Vec<_>) = $ys.into_iter().map(|(i, x)| (i as f32, x.clone() as f32)).unzip();
            let trace = Scatter::new(xs, ys)
                .mode(Mode::Lines)
                .name(name);
            plot.add_trace(trace);
        )+
        plot.set_layout(layout);
        plot.show();
    }}
}

