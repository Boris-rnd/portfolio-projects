use busy_beaver::*;
use std::time::{Duration, Instant};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let theory = [0, 1, 6, 21, 107, 47_176_870]; // Fifth is bigger than, but i'm not gonna get it 🤣
    for i in 2..4 {
        print!("Attemping busy beaver {i}... ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let (res, time) = time_fn(|| busy_beaver(i, theory[i]));
        // assert_eq!(res, theory[i]); // ChatGPT gave me this formula
        let potential_machines = (4 * i).pow((2 * i) as u32);
        let time_per_machine = time.as_nanos() / potential_machines as u128;
        println!("Done in {time:?} ({potential_machines}, {time_per_machine}ns)");
    }
}

pub fn time_fn<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    (f(), start.elapsed())
}
// #[library_benchmark]
