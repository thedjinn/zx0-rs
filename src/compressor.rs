use crate::{
    MAX_OFFSET_ZX0,
    MAX_OFFSET_ZX7
};

use crate::compress::{Block, compress};
use crate::optimize::optimize;

/// A struct containing a vector representing the compressed data, as well as metadata related to
/// the compression operation.
pub struct CompressionResult {
    /// A vector containing the compressed output data.
    pub output: Vec<u8>,

    /// This value represents the minimum gap that should be maintained between the compressed
    /// data's end address and the uncompressed data's end address when decompressing in-place.
    /// When using the backwards compression mode the gap has to be between the start of the
    /// compressed data and the start of the uncompressed data.
    ///
    /// Please refer to the original C implementation's
    /// [readme](https://github.com/einar-saukas/ZX0#compressing-with-prefix) for an in-depth
    /// explanation.
    pub delta: usize
}

pub type ProgressCallback<'a> = Box<dyn FnMut(f32) + 'a>;

/// This struct provides a means of initializing and performing a ZX0 compression operation by
/// leveraging the builder pattern.
///
/// By calling [`Compressor::new`] a new [`Compressor`] will be instantiated using the following
/// default values:
///
/// - No prefix/suffix skipping
/// - Quick mode disabled
/// - Backwards mode disabled
/// - Classic mode disabled
///
/// After constructing a [`Compressor`] instance the method [`compress`](Compressor::compress) is available to compress
/// `u8` slices. The [`Compressor`] can be resued again afterwards.
///
/// In contrast to the original C implementation, compression using the Rust ZX0 compressor is
/// thread-safe, and can therefore be used to compress several slices in parallel.
pub struct Compressor<'a> {
    skip: usize,
    quick_mode: bool,
    backwards_mode: bool,
    classic_mode: bool,
    progress_callback: ProgressCallback<'a>
}

impl<'a> Compressor<'a> {
    /// Instantiate a new [`Compressor`] using the following default values:
    ///
    /// - No prefix/suffix skipping
    /// - Quick mode disabled
    /// - Backwards mode disabled
    /// - Classic mode disabled
    pub fn new() -> Self {
        Self {
            skip: 0,
            quick_mode: false,
            backwards_mode: false,
            classic_mode: false,
            progress_callback: Box::new(|_| ())
        }
    }

    /// Change the value for the quick mode setting. When enabled, this will cause the ZX0
    /// compressor to use a smaller dictionary size, at the cost of a less efficient compression
    /// ratio.
    ///
    /// Enabling this setting can be useful when producing debug assets where a short feedback loop
    /// is more important than getting a good compression ratio.
    pub fn quick_mode(&mut self, quick_mode: bool) -> &mut Self {
        self.quick_mode = quick_mode;
        self
    }

    /// Change the value for the backwards compression mode setting. This will cause the ZX0
    /// compressor to create compressed data that should be decompressed back-to-front. This can be
    /// useful in situations where in-place decompression is desired, and the end of the compressed
    /// data overlaps with the end of the region that the uncompressed data should be positioned
    /// in.
    ///
    /// Please refer to the original C implementation's
    /// [readme](https://github.com/einar-saukas/ZX0#compressing-backwards) for an in-depth
    /// explanation.
    pub fn backwards_mode(&mut self, backwards_mode: bool) -> &mut Self {
        self.backwards_mode = backwards_mode;
        self
    }

    /// Change the value for the classic compression mode setting. Enabling this will cause the ZX0
    /// compressor to output compressed data in its legacy V1 file format. This can be useful when
    /// compressing for one of the platforms that only provides a V1 decompression routine.
    pub fn classic_mode(&mut self, classic_mode: bool) -> &mut Self {
        self.classic_mode = classic_mode;
        self
    }

    /// Set a progress callback. When providing a closure to this function, that closure will be
    /// called repeatedly during compression. The closure will be called with a progress value
    /// between `0.0` and `1.0`. Note that due to the nature of the compression algorithm, this
    /// value is not increasing linearly with time, and thus should be interpreted as a rough
    /// estimate.
    pub fn progress_callback<C: FnMut(f32) + 'a>(&mut self, progress_callback: C) -> &mut Self {
        self.progress_callback = Box::new(progress_callback);
        self
    }

    /// Set the number of prefix/suffix bytes to skip during compression. This will cause the
    /// compressor to create a dictionary based on data that will already be in memory before the
    /// compressed data during decompression. Of course, for this to work the prefix (or suffix in
    /// case of backwards mode) in the file must be 100% identical to the data that is in memory
    /// before or after the block of compressed data when attempting to decompress it.
    ///
    /// Please refer to the original C implementation's
    /// [readme](https://github.com/einar-saukas/ZX0#compressing-with-prefix) for an in-depth
    /// explanation.
    pub fn skip(&mut self, skip: usize) -> &mut Self {
        self.skip = skip;
        self
    }

    /// Compress the provided slice.
    ///
    /// This returns a [`CompressionResult`] struct containing both the compressed data as well as
    /// metadata related to the compression operation.
    ///
    /// The [`Compressor`] does not have to be discarded after calling this method. It does not
    /// contain any state (only the configuration) and thus can be reused again for compressing
    /// additional data.
    pub fn compress(&mut self, input: &[u8]) -> CompressionResult {
        let chain = {
            let (allocator, mut optimal) = optimize(
                input,
                self.skip,
                if self.quick_mode { MAX_OFFSET_ZX7 } else { MAX_OFFSET_ZX0 },
                &mut self.progress_callback
            );

            let mut chain = Vec::new();

            while optimal != 0 {
                let oblock = allocator.get(optimal);

                chain.push(Block {
                    bits: oblock.bits as u32,
                    index: oblock.index as isize,
                    offset: oblock.offset as usize
                });

                optimal = oblock.next_index;
            }

            chain
        };

        let invert_mode = !self.classic_mode && !self.backwards_mode;
        let mut delta = 0;

        let output = compress(
            chain,
            input,
            self.skip,
            self.backwards_mode,
            invert_mode,
            &mut delta
        );

        CompressionResult {
            output,
            delta
        }
    }
}

impl<'a> Default for Compressor<'a> {
    fn default() -> Self {
        Self::new()
    }
}
