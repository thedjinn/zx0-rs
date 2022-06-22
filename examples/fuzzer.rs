//! This example contains a fuzzer that feeds the reference compressor and the Rust-based
//! compressor with random data and compares the results.

use rand::{thread_rng, Rng};

fn main() {
    let mut input = vec![0u8; 2048];

    loop {
        thread_rng().fill(&mut input[..]);

        let reference = reference::Compressor::new().compress(&input);
        let result = zx0::Compressor::new().compress(&input);

        if result.output != reference.output {
            println!("Bad input: {:?}", input);
            println!("Reference: {:?}", reference.output);
            println!("Output:    {:?}", result.output);

            panic!("Output and reference don't match!");
        }
    }
}
