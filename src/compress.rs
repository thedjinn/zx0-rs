use crate::INITIAL_OFFSET;

pub struct Block {
    pub bits: u32,
    pub index: isize,
    pub offset: usize
}

struct Context<'a> {
    backtrack: bool,
    bit_mask: u8,
    bit_index: usize,
    input_index: usize,
    output: &'a mut Vec<u8>,
    output_index: usize,
    diff: isize
}

impl Context<'_> {
    fn read_bytes(
        &mut self,
        n: usize,
        delta: &mut usize
    ) {
        self.input_index += n;
        self.diff += n as isize;

        // Since delta starts with zero and is only written to here, it can be proven that it will
        // always be a non-negative number. Therefore its type is usize instead of diff's isize
        // (since it can become negative).
        if (*delta as isize) < self.diff {
            *delta = self.diff as usize;
        }
    }

    fn write_byte(
        &mut self,
        value: u8,
    ) {
        self.output[self.output_index] = value;
        self.output_index += 1;
        self.diff -= 1;
    }

    fn write_bit(
        &mut self,
        value: u8
    ) {
        if self.backtrack {
            if value != 0 {
                self.output[self.output_index - 1] |= 1;
            }

            self.backtrack = false;
        } else {
            if self.bit_mask == 0 {
                self.bit_mask = 128;
                self.bit_index = self.output_index;
                self.write_byte(0);
            }

            if value != 0 {
                self.output[self.bit_index] |= self.bit_mask as u8;
            }

            self.bit_mask >>= 1;
        }
    }

    fn write_interlaced_elias_gamma(
        &mut self,
        value: usize, // usize because the only values we will be writing are derived from it
        backwards_mode: bool,
        invert_mode: bool
    ) {
        let mut i = 2;

        while i <= value {
            i <<= 1;
        }

        i >>= 1;

        loop {
            i >>= 1;
            if i == 0 {
                break;
            }

            self.write_bit(
                if backwards_mode { 1 } else { 0 }
            );

            self.write_bit(
                if invert_mode {
                    if value & i != 0 {
                        0
                    } else {
                        1
                    }
                } else if value & i != 0 {
                    1
                } else {
                    0
                }
            );
        }

        self.write_bit(
            if !backwards_mode { 1 } else { 0 }
        );
    }
}

pub fn compress(
    chain: Vec<Block>,
    input: &[u8],
    skip: usize,
    backwards_mode: bool,
    invert_mode: bool,
    delta: &mut usize
) -> Vec<u8> {
    // Calculate and allocate output buffer
    let output_size = ((chain[0].bits + 25) / 8) as usize;
    let mut output = vec![0; output_size];

    // Un-reverse optimal sequence
    let chain = chain.into_iter().rev().collect::<Vec<_>>();

    // Initialize data
    let mut last_offset = INITIAL_OFFSET as usize;

    let mut context = Context {
        backtrack: true,
        bit_mask: 0,
        bit_index: 0,
        input_index: skip,
        output: &mut output,
        output_index: 0,
        // Note: this is normally a negative number, unless optimize() has a compression ratio <1.
        diff: output_size as isize - input.len() as isize + skip as isize
    };

    // Generate output
    for (previous_block, current_block) in chain.windows(2).map(|s| (&s[0], &s[1])) {
        let length = (current_block.index - previous_block.index) as usize;

        if current_block.offset == 0 {
            // Copy literals indicator
            context.write_bit(0);

            // Copy literals length
            context.write_interlaced_elias_gamma(length, backwards_mode, false);

            // Copy literals values
            for _ in 0..length {
                let byte = input[context.input_index];
                context.write_byte(byte);
                context.read_bytes(1, delta);
            }
        } else if current_block.offset == last_offset {
            // Copy from last offset indicator
            context.write_bit(0);

            // Copy from last offset length
            context.write_interlaced_elias_gamma(length, backwards_mode, false);
            context.read_bytes(length, delta);
        } else {
            // Copy from new offset indicator
            context.write_bit(1);

            // Copy from new offset MSB
            context.write_interlaced_elias_gamma((current_block.offset - 1) / 128 + 1, backwards_mode, invert_mode);

            // Copy from new offset LSB
            if backwards_mode {
                context.write_byte((((current_block.offset - 1) % 128) << 1) as u8);
            } else {
                context.write_byte(((127 - (current_block.offset - 1) % 128) << 1) as u8);
            }

            // Copy from new offset length */
            context.backtrack = true;
            context.write_interlaced_elias_gamma(length - 1, backwards_mode, false);
            context.read_bytes(length, delta);

            last_offset = current_block.offset;
        }
    }

    // End marker
    context.write_bit(1);
    context.write_interlaced_elias_gamma(256, backwards_mode, invert_mode);

    // Done
    output
}
