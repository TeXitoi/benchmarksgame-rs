// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by Cristi Cobzarenco (@cristicbz)

extern crate rayon;

use std::{cmp, mem};
use rayon::prelude::*;

const NUM_BLOCKS: u32 = 24;

fn fannkuch(n: i32) -> (i32, i32) {
    let mut factorials = [1; 16];
    for i in 1 .. n as usize + 1 {
        factorials[i] = factorials[i - 1] * i as u32;
    }

    let perm_max = factorials[n as usize];
    let (num_blocks, block_size) = if perm_max < NUM_BLOCKS {
        (1, perm_max)
    } else {
        (NUM_BLOCKS + if perm_max % NUM_BLOCKS == 0 { 0 } else { 1 },
         perm_max / NUM_BLOCKS)
    };

    (0..num_blocks).into_par_iter().map(|i_block| i_block * block_size).map(|initial| {
        let mut count = [0i32; 16];
        let mut temp = [0i32; 16];
        let mut current = [0i32; 16];
        for (i, value) in current.iter_mut().enumerate() {
            *value = i as i32;
        }

        let mut permutation_index = initial as i32;
        for i in (1..n as usize).rev() {
            let factorial = factorials[i] as i32;
            let d = permutation_index / factorial;
            permutation_index %= factorial;
            count[i] = d;

            temp.copy_from_slice(&current);
            let d = d as usize;
            for j in 0..i + 1 {
                current[j] = if j + d <= i { temp[j + d] } else { temp[j + d - i - 1] };
            }
        }

        let last_permutation_in_block = cmp::min(initial + block_size, perm_max) - 1;
        let mut permutation_index = initial;
        let (mut checksum, mut maxflips) = (0, 0);
        loop {
            if current[0] > 0 {
                let mut flip_count = 1;
                temp.copy_from_slice(&current);

                let mut first_value = current[0] as usize;
                while temp[first_value] != 0 {
                    let new_first_value = mem::replace(&mut temp[first_value], first_value as i32);
                    if first_value > 2 {
                        temp[1..first_value].reverse();
                    }
                    first_value = new_first_value as usize;
                    flip_count += 1;
                }

                checksum += if permutation_index % 2 == 0 { flip_count } else { -flip_count };
                maxflips = cmp::max(maxflips, flip_count);
            }

            if permutation_index >= last_permutation_in_block {
                break;
            }
            permutation_index += 1;

            let mut first_value = current[1];
            current[1] = current[0];
            current[0] = first_value;
            let mut i = 1;
            while count[i] >= i as i32 {
                count[i] = 0;
                i += 1;
                let new_first_value = current[1];
                current[0] = new_first_value;
                for j in 1..i {
                    current[j] = current[j + 1];
                }
                current[i] = mem::replace(&mut first_value, new_first_value);
            }
            count[i] += 1;
        }

        (checksum, maxflips)
    }).reduce(|| (0, 0), |(cs1, mf1), (cs2, mf2)| (cs1 + cs2, cmp::max(mf1, mf2)))
}

fn main() {
    let n = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(7);

    let (checksum, maxflips) = fannkuch(n);
    println!("{}\nPfannkuchen({}) = {}", checksum, n, maxflips);
}
