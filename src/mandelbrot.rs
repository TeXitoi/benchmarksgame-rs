// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by Matt Watson
use std::io::Write;
use std::io;
use std::thread;
const THREADS: usize = 8;
const MAX_ITER: usize = 50;
const DX: f64 = -1.5;
const DY: f64 = -1.0;
pub fn mbrotpt(x: f64, y: f64) -> usize {
    let mut z = (0.0, 0.0);
    for _ in 0..MAX_ITER {
        z = (z.0 * z.0 - z.1 * z.1 + x,
             2.0 * z.0 * z.1 + y);
        if z.0 * z.0 + z.1 * z.1 >= 4.0 {
            return 0;
        }
    }
    return 1;
}

fn mbrot8(x: usize, y: usize, inv: f64) -> u8 {
    let mut result = 0 as usize;
    let mut i = 0;
    while i < 8 {
        result = result << 1;
        result = result | mbrotpt((x + i) as f64 * inv + DX,
                                       y as f64 * inv + DY);
        i += 1;
    }
    result as u8
}

fn main() {
    let size = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(200);
    let inv = 2.0 / size as f64;
    println!("P4");
    println!("{} {}",size, size);
    let workers: Vec<usize> = (0..THREADS).collect();;
    let handles: Vec<_> = workers.into_iter().map(|t| {
        thread::spawn(move || {
            let mut rows = vec![vec![0 as u8; 8 * size / 64]; size / THREADS];
            for z in 0..size / THREADS {
                let mut row = vec![0; size / 8];
                for x in 0..size / 8 {
                    row[x] = mbrot8(x * 8,t * (size / THREADS) + z, inv);
                }
                rows[z] = row.to_vec();
            }
            rows
        })
    }).collect();

    for h in handles {
        let rows = h.join().unwrap();
        for i in 0..size / THREADS {
            std::io::stdout().write(&rows[i]).ok().expect("Could not write to stdout");
        }
    }
    io::stdout().flush().ok().expect("Could not flush stdout");
}
