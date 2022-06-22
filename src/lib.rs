#![warn(missing_docs)]

//! A ZX0 compressor implementation for Rust.
//!
//! This crate provides a Rust implementation for Einar Saukas' excellent ZX0 compression
//! algorithm.
//!
//! The algorithm provided in this crate is a more optimized variant of the original C-based
//! implementation, and is therefore about 40% faster compared to the original. Additionally, the
//! Rust implementation also offers thread-safety, meaning that files can now be compressed in
//! parallel. Finally, this implementation is also free of memory leaks.
//!
//! To guarantee correctness the crate offers a sub-crate containing a Rust wrapper of the original
//! C code. This wrapper is used as a reference in the crate's test suite to ensure that its output
//! is 100% equivalent to the original implementation.
//!
//! The compressor can be used in two ways:
//!
//! 1. By instantiating a [`Compressor`] instance, configuring it, and invoking its
//!    [`compress`](Compressor::compress) method.
//!
//! 2. Using the top level [`compress`](compress()) shortcut function to compress with the default settings.
//!
//! Please refer to the documentation for the [`Compressor`] struct for more information on how to
//! use this crate, or inspect the examples that are provided in the crate's source code.
//!
//! Additionally, there is a wealth of information provided in the readme file of Einar Saukas'
//! original implementation.

mod compress;
mod compressor;
mod optimize;

const INITIAL_OFFSET: usize = 1;
const MAX_OFFSET_ZX0: usize = 32640;
const MAX_OFFSET_ZX7: usize = 2176;

pub use compressor::{
    CompressionResult,
    Compressor
};

/// Compress the input slice to an output vector.
///
/// This is a shortcut for:
///
/// ```rust
/// Compressor::new().compress(input).output
/// ```
///
/// For a more customized experience please see the [`Compressor`] struct.
pub fn compress(input: &[u8]) -> Vec<u8> {
    Compressor::new().compress(input).output
}

#[cfg(test)]
mod tests {
    use super::Compressor;

    #[test]
    fn defaults() {
        let input = std::fs::read("src/lib.rs").unwrap();

        let reference = reference::Compressor::new().compress(&input);
        let result = Compressor::new().compress(&input);

        assert_eq!(result.output, reference.output);
        assert_eq!(result.delta, reference.delta);
    }

    #[test]
    fn defaults_with_prefix() {
        let input = std::fs::read("src/lib.rs").unwrap();

        // This may take a minute on a debug build
        for skip in (0..input.len()).step_by(512) {
            let reference = reference::Compressor::new().skip(skip).compress(&input);
            unsafe { reference::reset(); }

            let result = Compressor::new().skip(skip).compress(&input);

            assert_eq!(result.output, reference.output);
            assert_eq!(result.delta, reference.delta);
        }
    }

    #[test]
    fn backwards_mode() {
        let input = std::fs::read("src/lib.rs").unwrap();

        let reference = reference::Compressor::new().backwards_mode(true).compress(&input);
        unsafe { reference::reset(); }

        let result = Compressor::new().backwards_mode(true).compress(&input);

        assert_eq!(result.output, reference.output);
        assert_eq!(result.delta, reference.delta);
    }

    #[test]
    fn backwards_mode_with_suffix() {
        let input = std::fs::read("src/lib.rs").unwrap();

        // This may take a minute on a debug build
        for skip in (0..input.len()).step_by(512) {
            let reference = reference::Compressor::new().backwards_mode(true).skip(skip).compress(&input);
            unsafe { reference::reset(); }

            let result = Compressor::new().backwards_mode(true).skip(skip).compress(&input);

            assert_eq!(result.output, reference.output);
            assert_eq!(result.delta, reference.delta);
        }
    }

    #[test]
    fn quick_mode() {
        let input = std::fs::read("src/lib.rs").unwrap();

        let reference = reference::Compressor::new().quick_mode(true).compress(&input);
        unsafe { reference::reset(); }

        let result = Compressor::new().quick_mode(true).compress(&input);

        assert_eq!(result.output, reference.output);
        assert_eq!(result.delta, reference.delta);
    }

    #[test]
    fn classic_mode() {
        let input = std::fs::read("src/lib.rs").unwrap();

        let reference = reference::Compressor::new().classic_mode(true).compress(&input);
        unsafe { reference::reset(); }

        let result = Compressor::new().classic_mode(true).compress(&input);

        assert_eq!(result.output, reference.output);
        assert_eq!(result.delta, reference.delta);
    }

    #[test]
    fn progress_callback() {
        let input = std::fs::read("src/lib.rs").unwrap();

        let called = std::cell::RefCell::new(false);

        Compressor::new().progress_callback(|progress| {
            *called.borrow_mut() = true;
            assert!(progress >= 0.0);
            assert!(progress <= 1.0);
        }).compress(&input);

        assert!(*called.borrow());
    }
}
