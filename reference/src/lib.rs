use std::slice;
use std::sync::Mutex;

use once_cell::sync::Lazy;

mod bindings {
    #![allow(clippy::upper_case_acronyms)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// For fair benchmarks
pub use bindings::reset;

// The original C implementation is not thread safe.
static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

const MAX_OFFSET_ZX0: i32 = 32640;
const MAX_OFFSET_ZX7: i32 = 2176;

pub struct CompressionResult {
    pub output: Vec<u8>,
    pub delta: usize
}

pub struct Compressor {
    skip: usize,
    quick_mode: bool,
    backwards_mode: bool,
    classic_mode: bool
}

impl Compressor {
    pub fn new() -> Self {
        Self {
            skip: 0,
            quick_mode: false,
            backwards_mode: false,
            classic_mode: false
        }
    }

    pub fn quick_mode(&mut self, quick_mode: bool) -> &mut Self {
        self.quick_mode = quick_mode;
        self
    }

    pub fn backwards_mode(&mut self, backwards_mode: bool) -> &mut Self {
        self.backwards_mode = backwards_mode;
        self
    }

    pub fn classic_mode(&mut self, classic_mode: bool) -> &mut Self {
        self.classic_mode = classic_mode;
        self
    }

    pub fn skip(&mut self, skip: usize) -> &mut Self {
        self.skip = skip;
        self
    }

    pub fn compress(&self, input: &[u8]) -> CompressionResult {
        let invert_mode = !self.classic_mode && !self.backwards_mode;

        let mut delta = 0;

        let output = unsafe {
            let _lock = LOCK.lock().unwrap();

            let mut output_size = 0;

            let output_data = bindings::compress(
                bindings::optimize(
                    input.as_ptr(),
                    input.len() as i32,
                    self.skip as i32,
                    if self.quick_mode { MAX_OFFSET_ZX7 } else { MAX_OFFSET_ZX0 }
                ),
                input.as_ptr(),
                input.len() as i32,
                self.skip as i32,
                if self.backwards_mode { 1 } else { 0 },
                if invert_mode { 1 } else { 0 },
                &mut output_size,
                &mut delta
            );

            slice::from_raw_parts(output_data, output_size as usize).to_vec()
        };

        CompressionResult {
            output,
            delta: delta as usize
        }
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Compressor};

    #[test]
    fn compare_defaults() {
        let input = std::fs::read("src/lib.rs").unwrap();

        let result = Compressor::new().compress(&input);

        assert!(result.output.len() > 0);
    }
}
