use std::collections::VecDeque;

use crate::INITIAL_OFFSET;
use crate::compressor::ProgressCallback;

fn offset_ceiling(index: usize, offset_limit: usize) -> usize {
    if index > offset_limit {
        offset_limit
    } else if index < INITIAL_OFFSET {
        INITIAL_OFFSET
    } else {
        index
    }
}

/// Compute the number of bits required to represent the Elias Gamma code for the specified value.
///
/// This computes 2 * floor(log2(value)) + 1.
fn elias_gamma_bits(value: u32) -> u32 {
    2 * (u32::BITS - value.leading_zeros() - 1) + 1
}

pub struct Block {
    pub bits: u32,
    pub index: i32,
    pub offset: u32,
    pub next_index: usize,
    refcount: u32
}

pub struct Allocator {
    free_list: VecDeque<u32>,
    blocks: Vec<Block>
}

impl Allocator {
    fn new() -> Self {
        Self {
            free_list: VecDeque::new(),
            blocks: {
                let mut blocks = Vec::with_capacity(1024 * 1024);

                // Special block index only used for null values
                blocks.push(Block {
                    bits: 0, index: 0, offset: 0, next_index: 0, refcount: 0
                });

                blocks
            }
        }
    }

    // Assign *ptr to chain and update refcounts, adding the old value of ptr to the free list if
    // its refcount reaches zero.
    #[inline(always)]
    fn assign(&mut self, ptr: &mut usize, next_index: usize) {
        self.blocks[next_index].refcount += 1;

        if *ptr != 0 {
            self.blocks[*ptr].refcount -= 1;

            if self.blocks[*ptr].refcount == 0 {
                self.free_list.push_back(*ptr as u32);
            }
        }

        *ptr = next_index;
    }

    #[inline(always)]
    fn assign_new(&mut self, ptr: &mut usize, bits: u32, index: i32, offset: u32, next_index: usize) {
        if next_index != 0 {
            self.blocks[next_index].refcount += 1;
        }

        if *ptr != 0 {
            self.blocks[*ptr].refcount -= 1;

            if self.blocks[*ptr].refcount == 0 {
                self.free_list.push_back(*ptr as u32);
            }
        }

        let block = Block {
            bits,
            index,
            offset,
            next_index,
            refcount: 1
        };

        *ptr = if let Some(i) = self.free_list.pop_front() {
            self.blocks[i as usize] = block;

            i as usize
        } else {
            // Nothing in the free list, allocate a block
            self.blocks.push(block);
            self.blocks.len() - 1
        };
    }

    pub fn get(&self, index: usize) -> &Block {
        &self.blocks[index]
    }
}

pub fn optimize(
    input: &[u8],
    skip: usize,
    offset_limit: usize,
    progress_callback: &mut ProgressCallback
) -> (Allocator, usize) {
    let mut allocator = Allocator::new();

    let max_offset = offset_ceiling(input.len() - 1, offset_limit);

    // Allocate all main data structures at once
    let mut last_literal = vec![0; max_offset + 1];
    let mut last_match = vec![0; max_offset + 1];
    let mut optimal = vec![0; input.len()];
    let mut match_length: Vec<usize> = vec![0; max_offset + 1];
    let mut best_length: Vec<usize> = vec![0; input.len()];

    if input.len() > 2 {
        best_length[2] = 2;
    }

    // Start with fake block
    allocator.assign_new(
        &mut last_match[INITIAL_OFFSET],
        0,
        skip as i32 - 1,
        INITIAL_OFFSET as u32,
        0
    );

    // Process remaining bytes
    for index in skip..input.len() {
        if index % 128 == 0 {
            progress_callback(index as f32 / (input.len() - skip) as f32);
        }

        let mut best_length_size = 2;
        let max_offset = offset_ceiling(index, offset_limit);

        for offset in 1..=max_offset {
            if index >= offset && index != skip && input[index] == input[index - offset] {
                // Copy from last offset
                if last_literal[offset] != 0 {
                    let length = index as i32 - allocator.get(last_literal[offset]).index;
                    let bits = allocator.get(last_literal[offset]).bits + 1 + elias_gamma_bits(length as u32);

                    allocator.assign_new(
                        &mut last_match[offset],
                        bits, index as i32, offset as u32, last_literal[offset]
                    );

                    if optimal[index] == 0 || allocator.get(optimal[index]).bits > bits {
                        allocator.assign(&mut optimal[index], last_match[offset]);
                    }
                }

                // Copy from new offset
                match_length[offset] += 1;
                if match_length[offset] > 1 {
                    if best_length_size < match_length[offset] {
                        let mut bits = allocator.get(optimal[index - best_length[best_length_size]]).bits + elias_gamma_bits(best_length[best_length_size] as u32 - 1);

                        loop {
                            best_length_size += 1;
                            let bits2 = allocator.get(optimal[index - best_length_size]).bits + elias_gamma_bits(best_length_size as u32 - 1);

                            if bits2 <= bits {
                                best_length[best_length_size] = best_length_size;
                                bits = bits2;
                            } else {
                                best_length[best_length_size] = best_length[best_length_size as usize - 1];
                            }

                            if best_length_size >= match_length[offset] {
                                break;
                            }
                        }
                    }

                    let length = best_length[match_length[offset]];
                    let bits = allocator.get(optimal[index - length]).bits + 8 + elias_gamma_bits((offset as u32 - 1) / 128 + 1) + elias_gamma_bits(length as u32 - 1);

                    if last_match[offset] == 0 || allocator.get(last_match[offset]).index != index as i32 || allocator.get(last_match[offset]).bits > bits {
                        allocator.assign_new(
                            &mut last_match[offset],
                            bits, index as i32, offset as u32, optimal[index - length as usize]
                        );

                        if optimal[index] == 0 || allocator.get(optimal[index]).bits > bits {
                            allocator.assign(&mut optimal[index], last_match[offset]);
                        }
                    }
                }
            } else {
                // Copy literals
                match_length[offset] = 0;

                if last_match[offset] != 0 {
                    let length = index as i32 - allocator.get(last_match[offset]).index;
                    let bits = allocator.get(last_match[offset]).bits + 1 + elias_gamma_bits(length as u32) + length as u32 * 8;

                    allocator.assign_new(
                        &mut last_literal[offset],
                        bits, index as i32, 0, last_match[offset]
                    );

                    if optimal[index] == 0 || allocator.get(optimal[index]).bits > bits {
                        allocator.assign(&mut optimal[index], last_literal[offset]);
                    }
                }
            }
        }
    }

    (allocator, optimal[input.len() - 1])
}
