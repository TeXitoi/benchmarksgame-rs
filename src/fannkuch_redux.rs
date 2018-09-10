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

pub fn pack(perm: &[u8; 16]) -> u64 {
    perm.iter().rev().fold(0, |acc, &i| (acc << 4) + i as u64)
}
pub fn flips(perm: u64) -> i32 {
    const LOWER: u64 = 0x0f0f0f0f0f0f0f0fu64;
    let (mut flip, mut flip_count) = (perm, 0);
    loop {
        let flip_index = (flip & 0xf) as usize;
        if flip_index == 0 { break; }
        let (s, n4) = (flip.swap_bytes(), flip_index * 4);
        flip &= !0xf << n4;
        flip |= (((s & LOWER) << 4) | (s >> 4) & LOWER) >> (60 - n4);
        flip_count += 1;
    }
    flip_count
}
pub fn permute(perm: u64, count: &mut [u8; 16]) -> u64 {
    let mut perm = rotate(perm, 1);
    // Generate the next permutation.
    let mut i = 1;
    while count[i] >= i as u8 {
        count[i] = 0;
        i += 1;
        perm = rotate(perm, i);
    }
    count[i] += 1;
    perm
}
fn rotate(perm: u64, n: usize) -> u64 {
    let n4 = n * 4;
    let mask = !0xf << n4;
    perm & mask | (perm & !mask) >> 4 | (perm & 0xf) << n4
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

    // Compute the `checksum` and `maxflips` for each block in parallel.
    (0..num_blocks).into_par_iter().map(|i_block| {
        let initial = i_block * block_size;
        let mut count = [0u8; 16];
        let mut temp = [0u8; 16];
        let mut current = [0u8; 16];

        // Initialise `count` and the current permutation (`current`)
        current.iter_mut().enumerate().for_each(|(i, value)| *value = i as u8);

        let mut permutation_index = initial as i32;
        for i in (1..n as usize).rev() {
            let factorial = factorials[i] as i32;
            let d = permutation_index / factorial;
            permutation_index %= factorial;
            count[i] = d as u8;

            temp.copy_from_slice(&current);
            let d = d as usize;
            current[0..=i - d].copy_from_slice(&temp[d..=i]);
            current[i - d + 1..=i].copy_from_slice(&temp[0..d])
        }

        // Iterate over each permutation in the block.
        let mut perm = pack(&current);
        let last_permutation_in_block = cmp::min(initial + block_size,
                                                 perm_max) - 1;
        let mut permutation_index = initial;
        let (mut checksum, mut maxflips) = (0, 0);
        loop {
            // If the first value in the current permutation is not 1 (0) then
            // we will need to do at least one flip for `current`.
            if perm & 0xf > 0 {
                let flip_count = flips(perm);
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
            perm = permute(perm, &mut count);
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
