// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by Cristi Cobzarenco (@cristicbz)
// contributed by Andre Bogus

extern crate rayon;

use std::cmp;
use rayon::prelude::*;
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone)]
pub struct U8x16(__m128i);

#[cfg(all(target_arch = "x86_64", target_feature = "sse2",
          target_feature = "ssse3"))]
impl U8x16 {
    pub fn zero() -> U8x16 { U8x16(unsafe { _mm_setzero_si128() }) }
    pub fn from_slice_unaligned(s: &[u8; 16]) -> U8x16 {
        U8x16(unsafe { _mm_loadu_si128(s.as_ptr() as *const _) })
    }
    pub fn write_to_slice_unaligned(self, s: &mut [u8; 16]) {
        unsafe { _mm_storeu_si128(s.as_mut_ptr() as *mut _, self.0) }
    }
    pub fn extract0(self) -> i32 {
        unsafe { _mm_extract_epi16(self.0, 0i32) & 0xFF }
    }
    pub fn permute_dyn(self, indices: U8x16) -> U8x16 {
        U8x16(unsafe { _mm_shuffle_epi8(self.0, indices.0) })
    }
}

// This value controls the preferred maximum number of  blocks the workload is
// broken up into. The actual value may be one higher (if the number of
// permutations doesn't divide exactly by this value) or might be set to 1 if
// the number of permutations is lower than this value.
const NUM_BLOCKS: u32 = 24;

fn fannkuch(n: i32) -> (i32, i32) {
    // Precompute a table a factorials to reuse all over the place.
    let mut factorials = [1; 16];
    for i in 1..=n as usize {
        factorials[i] = factorials[i - 1] * i as u32;
    }
    let perm_max = factorials[n as usize];

    // Compute the number of blocks and their size. If n! is less than
    // NUM_BLOCKS then use a single block (perform the work serially for small
    // values of n). If n! doesn't divide exactly by NUM_BLOCKS, then add one
    // extra block to compute the remainder.
    let (num_blocks, block_size) = if perm_max < NUM_BLOCKS {
        (1, perm_max)
    } else {
        (NUM_BLOCKS + if perm_max % NUM_BLOCKS == 0 { 0 } else { 1 },
         perm_max / NUM_BLOCKS)
    };

    // precompute flips and rotations
    let mut flip_masks = [U8x16::zero(); 16];
    let mut rotate_masks = [U8x16::zero(); 16];
    let mut mask = [0u8; 16];
    for i in 0..16 {
        mask.iter_mut().enumerate().for_each(|(j, m)| *m = j as u8);
        mask[0..i + 1].reverse();
        flip_masks[i] = U8x16::from_slice_unaligned(&mask);
        mask.iter_mut().enumerate().for_each(|(j, m)| *m = j as u8);
        let c = mask[0];
        (0..i).for_each(|i| mask[i] = mask[i + 1]);
        mask[i] = c;
        rotate_masks[i] = U8x16::from_slice_unaligned(&mask);
    }

    // Compute the `checksum` and `maxflips` for each block in parallel.
    (0..num_blocks).into_par_iter().map(|i_block| {
        let initial = i_block * block_size;
        let mut count = [0i32; 16];
        let mut temp = [0u8; 16];
        let mut current = [0u8; 16];

        // Initialise `count` and the current permutation (`current`)
        current.iter_mut().enumerate().for_each(|(i, value)| *value = i as u8);

        let mut permutation_index = initial as i32;
        for i in (1..n as usize).rev() {
            let factorial = factorials[i] as i32;
            let d = permutation_index / factorial;
            permutation_index %= factorial;
            count[i] = d;

            temp.copy_from_slice(&current);
            let d = d as usize;
            current[0..=i - d].copy_from_slice(&temp[d..=i]);
            current[i - d + 1..=i].copy_from_slice(&temp[0..d])
        }

        // Iterate over each permutation in the block.
        let mut perm = U8x16::from_slice_unaligned(&current);
        let last_permutation_in_block = cmp::min(initial + block_size,
                                                 perm_max) - 1;
        let mut permutation_index = initial;
        let (mut checksum, mut maxflips) = (0, 0);
        loop {
            // If the first value in the current permutation is not 1 (0) then
            // we will need to do at least one flip for `current`.
            if perm.extract0() > 0 {
                // Copy the current permutation to work on it.
                let mut flip_count = 0;
                let mut flip = perm;
                loop {
                    let flip_index = flip.extract0() as usize;
                    if flip_index == 0 { break; }
                    flip = flip.permute_dyn(flip_masks[flip_index]);
                    flip_count += 1;
                }

                // Update the `checksum` and `maxflips` of this block.
                checksum += if permutation_index % 2 == 0 {
                    flip_count
                } else {
                    -flip_count
                };
                maxflips = cmp::max(maxflips, flip_count);
            }

            // If this was the last permutation in the block, we're done: return
            // the `checksum` and `maxflips` values which get reduced across
            // blocks in parallel by `rayon`.
            if permutation_index >= last_permutation_in_block {
                return (checksum, maxflips);
            }
            permutation_index += 1;
            perm = perm.permute_dyn(rotate_masks[1]);
            // Generate the next permutation.
            let mut i = 1;
            while count[i] >= i as i32 {
                count[i] = 0;
                i += 1;
                perm = perm.permute_dyn(rotate_masks[i]);
            }
            count[i] += 1;
        }
    }).reduce(|| (0, 0),
              |(cs1, mf1), (cs2, mf2)| (cs1 + cs2, cmp::max(mf1, mf2)))
}

fn main() {
    let n = std::env::args().nth(1)
        .and_then(|n| n.parse().ok())
        .unwrap_or(7);

    let (checksum, maxflips) = fannkuch(n);
    println!("{}\nPfannkuchen({}) = {}", checksum, n, maxflips);
}
