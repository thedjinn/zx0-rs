//! This example runs several iterations of the reference compressor and the Rust-based compressor
//! and shows their aggregated running times.

use std::time::Instant;

fn benchmark<C, T>(closure: C) where C: Fn() -> T {
    println!("Starting benchmark");

    let durations = (0..10).map(|_| {
        let start = Instant::now();
        closure();
        let end = Instant::now();

        let duration = (end - start).as_secs_f32() * 1000.0;
        println!("\x1b[1;32mCOMPRESSED\x1b[0;32m in \x1b[92m{:.3}\x1b[32m ms\x1b[0m", duration);

        unsafe { reference::reset() };

        duration
    }).collect::<Vec<_>>();

    let mean = durations.iter().sum::<f32>() / 10.0;
    let min = durations.iter().cloned().reduce(|a, b| a.min(b)).unwrap();
    let max = durations.iter().cloned().reduce(|a, b| a.max(b)).unwrap();

    println!("Done, mean = {:.3}, min = {:.3}, max = {:.3}, spread = {:.3}", mean, min, max, max - min);
}

fn main() {
    let input = std::fs::read("src/lib.rs").unwrap();

    benchmark(|| reference::Compressor::new().compress(&input));
    benchmark(|| zx0::Compressor::new().compress(&input));
}
