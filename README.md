# A ZX0 compressor implementation for Rust

This crate provides a Rust implementation for Einar Saukas' excellent ZX0
compression algorithm.

The algorithm provided in this crate is a more optimized variant of the
original C-based implementation, and is therefore about 40% faster compared to
the original. Additionally, the Rust implementation also offers thread-safety,
meaning that files can now be compressed in parallel. Finally, this
implementation is also free of memory leaks.

To guarantee correctness the crate offers a sub-crate containing a Rust
wrapper of the original C code. This wrapper is used as a reference in the
crate's test suite to ensure that its output is 100% equivalent to the
original implementation.

The compressor can be used in two ways:

1. By instantiating a `Compressor` instance, configuring it, and invoking its
   `compress` method.

2. Using the top level `compress` shortcut function to compress with the
   default settings.

Please refer to the documentation for the `Compressor` struct for more
information on how to use this crate, or inspect the examples that are
provided in the crate's source code.

Additionally, there is a wealth of information provided in the [readme
file](https://github.com/einar-saukas/ZX0#readme) of Einar Saukas' original
implementation.

## Usage

To start using the ZX0 compressor in your own projects, add the following line
to your Cargo dependencies:

```
zx0 = "1.0.0"
```

Then either invoke the compressor via the provided struct:

```rust
use zx0::Compressor;

let result = Compressor::new().compress(input_slice);

// From here you can access the compressed data with result.output, and
// retrieve any compressor metadata such as the "delta" value by accessing the
// other struct members.
```

Alternatively, if all you need to do is compress some data and use the
compressed output data somewhere, you can use this handy shortcut:

```rust
let output_vec = zx0::compress(input_slice);
```

## Advanced usage

The `Compressor` struct provides a builder-style configuration context. By
calling a few extra methods the compressor can be configured in exactly the
same way as the original C-based version.

```
use zx0::Compressor;

let result = Compressor::new()
    .skip(128)            // Prefix/suffix skipping
    .backwards_mode(true) // Backward compression
    .quick_mode(true)     // Quick but less efficient compression
    .classic_mode(true)   // V1 file format
    .compress(input_slice);
```

Additionally, a progress callback can be specified. This callback will be
invoked periodically during the compression process and will be provided with
a progress value ranging from `0.0` to `1.0`:

```
use zx0::Compressor;

let result = Compressor::new()
    .progress_callback(|progress| {
        println!("Compression progress: {:.2} percent", progress * 100.0);
    })
    .compress(input_slice);
```

For more information on how to use the skip and backwards mode features,
please refer to the [readme
file](https://github.com/einar-saukas/ZX0#readme) of Einar Saukas' original
implementation.

## License

As with the original C implementation, the compressor and all other code in
this crate is released under the 3-clause BSD License.
