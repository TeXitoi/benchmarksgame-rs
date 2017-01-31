// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by Matt Watson
// contributed by TeXitoi

use std::io::Write;
use std::thread;
use std::ops::{Add, Mul, Sub};

const THREADS: usize = 20;
const MAX_ITER: usize = 50;
const VLEN: usize = 8;
const ZEROS: Vecf64 = Vecf64([0.0; VLEN]);

macro_rules! for_vec {
    ( in_each [ $( $val:tt ),* ] do $from:ident $op:tt $other:ident ) => {
        $( $from.0[$val] $op $other.0[$val]; )*
    };
    ( $from:ident $op:tt $other:ident ) => {
        for_vec!(in_each [0, 1, 2, 3, 4, 5, 6, 7] do $from $op $other);
    };
}

#[derive(Clone, Copy)]
pub struct Vecf64([f64; VLEN]);
impl Mul for Vecf64 {
    type Output = Vecf64;
    fn mul(mut self, other: Vecf64) -> Vecf64 {
        for_vec!(self *= other);
        self
    }
}
impl Add for Vecf64 {
    type Output = Vecf64;
    fn add(mut self, other: Vecf64) -> Vecf64 {
        for_vec!(self += other);
        self
    }
}
impl Sub for Vecf64 {
    type Output = Vecf64;
    fn sub(mut self, other: Vecf64) -> Vecf64 {
        for_vec!(self -= other);
        self
    }
}

pub fn mbrot8(cr: Vecf64, ci: Vecf64) -> u8 {
    let mut zr = ZEROS;
    let mut zi = ZEROS;
    let mut tr = ZEROS;
    let mut ti = ZEROS;
    for _ in 0..MAX_ITER / 5 {
        for _ in 0..5 {
            zi = (zr + zr) * zi + ci;
            zr = tr - ti + cr;
            tr = zr * zr;
            ti = zi * zi;
        }
        if (tr + ti).0.iter().all(|&t| t > 4.) {
            return 0;
        }
    }
    (tr + ti).0.iter()
        .enumerate()
        .map(|(i, &t)| if t <= 4. { 0x80 >> i } else { 0 })
        .fold(0, |accu, b| accu | b)
}

fn main() {
    let size = std::env::args().nth(1)
        .and_then(|n| n.parse().ok())
        .unwrap_or(200);
    let inv = 2.0 / size as f64;
    let mut xvals = vec![0.0; size];
    let mut yvals = vec![0.0; size];
    for i in 0..size {
        xvals[i] = i as f64 * inv - 1.5;
        yvals[i] = i as f64 * inv - 1.0;
    }
    let xloc = &xvals;
    let yloc = &yvals;

    assert!(size % THREADS == 0);// FIXME
    let handles: Vec<_> = (0..THREADS).map(|e| {
        let xloc = xloc.to_vec();
        let yloc = yloc.to_vec();
        thread::spawn(move || {
            let mut rows = vec![vec![0 as u8; size / 8]; size / THREADS];
            for y in 0..size / THREADS {
                for x in 0..size / 8 {
                    let mut cr = ZEROS;
                    let ci = Vecf64([yloc[y + e * size / THREADS]; VLEN]);
                    for i in 0..VLEN {
                        cr.0[i] = xloc[8 * x + i];
                    }
                    rows[y][x] = mbrot8(cr, ci);
                }
            }
            rows
        })
    }).collect();

    println!("P4\n{} {}", size, size);
    let stdout_unlocked = std::io::stdout();
    let mut stdout = stdout_unlocked.lock();
    for row in handles.into_iter().flat_map(|h| h.join().unwrap().into_iter()) {
        stdout.write_all(&row).unwrap();
    }
    stdout.flush().unwrap();
}
