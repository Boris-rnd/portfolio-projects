// use plotters::{coord::ranged1d::AsRangedCoord, prelude::*};
// use std::fs;

// #[derive(Debug)]
// struct Entry {
//     func: String,
//     input: usize,
//     fastest: f64,
//     slowest: f64,
//     median: f64,
//     mean: f64,
// }
// #[track_caller]
// fn parse_time(s: &str) -> f64 {
//     if s.ends_with("ns") {
//         s.trim_end_matches(" ns").parse::<f64>().unwrap()
//     } else if s.ends_with("µs") {
//         s.trim_end_matches(" µs").parse::<f64>().unwrap() * 1e3
//     } else if s.ends_with("ms") {
//         s.trim_end_matches(" ms").parse::<f64>().unwrap() * 1e6
//     } else if s.ends_with("s") {
//         s.trim_end_matches(" s").parse::<f64>().unwrap() * 1e9
//     } else {
//         panic!("Unknown unit: {s}");
//     }
// }

// fn create_plot(
//     x_spec: std::ops::Range<usize>,
//     y_spec: std::ops::Range<usize>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let text = fs::read_to_string("divan.out")?.replace("╰─", "├─");
//     let mut entries = vec![];

//     let mut current_func = String::new();

//     for line in text.lines().skip(2) {
//         if line.starts_with("├─") {
//             // Function header
//             dbg!(line);
//             current_func = line
//                 .trim_start_matches(['├', '─', '╰', '│', ' '])
//                 .split_whitespace()
//                 .next()
//                 .unwrap()
//                 .to_string();
//         } else {
//             let line = line.trim().to_string();

//             // Data line
//             let line = line.replace("├─", "");
//             let line = line.trim().strip_prefix("│  ").unwrap_or(&line);
//             let line = line.trim();
//             let input: usize = line[0..line.find(" ").unwrap()].parse().unwrap();

//             let parts: Vec<_> = line[line.find(" ").unwrap() + 1..]
//                 .trim()
//                 .split('│')
//                 .map(|s| s.trim())
//                 .filter(|s| !s.is_empty())
//                 .collect();

//             if parts.len() < 5 {
//                 continue;
//             }

//             let fastest = parse_time(parts[0]);
//             let slowest = parse_time(parts[1]);
//             let median = parse_time(parts[2]);
//             let mean = parse_time(parts[3]);

//             entries.push(Entry {
//                 func: current_func.clone(),
//                 input,
//                 fastest,
//                 slowest,
//                 median,
//                 mean,
//             });
//             dbg!(entries.last().unwrap());
//         }
//     }

//     // Group by function
//     let mut funcs: Vec<String> = entries.iter().map(|e| e.func.clone()).collect();
//     funcs.sort();
//     funcs.dedup();
//     let root = BitMapBackend::new("plot.png", (1024, 768)).into_drawing_area();
//     root.fill(&WHITE)?;
//     let mut chart = ChartBuilder::on(&root)
//         .caption("Divan Results", ("sans-serif", 30))
//         .margin(10)
//         .x_label_area_size(40)
//         .y_label_area_size(60)
//         .build_cartesian_2d(x_spec, y_spec)?;

//     chart
//         .configure_mesh()
//         .y_desc("Mean time (log scale)")
//         .x_desc("Input size")
//         .y_label_formatter(&|y| format!("{:.2e}", y.exp()))
//         .draw()?;

//     // group by func
//     use std::collections::HashMap;
//     let mut groups: HashMap<String, Vec<&Entry>> = HashMap::new();
//     for e in &entries {
//         groups.entry(e.func.clone()).or_default().push(e);
//     }

//     for (i, (func, data)) in groups.into_iter().enumerate() {
//         let color = Palette99::pick(i).mix(0.9);
//         let series = data.iter().map(|e| (e.input, e.mean.ln()));
//         chart
//             .draw_series(LineSeries::new(series, &color))?
//             .label(func)
//             .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
//     }

//     chart
//         .configure_series_labels()
//         .border_style(&BLACK)
//         .draw()?;

//     Ok(())
// }

use std::collections::{BTreeMap, HashMap};

fn main() {
    let max_iter = 1_000_000;
    let mut kbonnaci_times = BTreeMap::new();
    let mut i = 1;
    let mut j = 0;
    while i < max_iter {
        if i < 10_000 {
            i += 500;
        } else {
            i = (i as f64 * 1.3) as usize;
        }
        let start = std::time::Instant::now();
        let _ = kbonaci::kbonacci(i, 1);
        let elapsed = start.elapsed().as_millis();
        kbonnaci_times.insert(i, elapsed);
        j += 1;
        if j % 4 == 0 {
            print!("{i}/{max_iter}\r");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    
    let mut fib_times = BTreeMap::new();

    for i in 0..40 {
        let start = std::time::Instant::now();
        let _ = kbonaci::fibonacci(i);
        let elapsed = start.elapsed().as_millis();
        fib_times.insert(i, elapsed);
    }
    
    dict_plot!("K-Bonacci benchmark", kbonnaci_times, fib_times);
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