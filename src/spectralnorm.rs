// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by Matt Brubeck
// contributed by TeXitoi
// contributed by Cristi Cobzarenco (@cristicbz)

#![allow(non_snake_case)]

extern crate rayon;
use rayon::prelude::*;

fn main() {
    let n = std::env::args().nth(1)
        .and_then(|n| n.parse().ok())
        .unwrap_or(100);
    let answer = spectralnorm(n);
    println!("{:.9}", answer);
}

fn spectralnorm(n: usize) -> f64 {
    assert!(n % 2 == 0, "only even lengths are accepted");
    let mut u = vec![1.0; n];
    let mut v = vec![0.0; n];
    let mut tmp = vec![0.0; n];
    for _ in 0..10 {
        mult_AtAv(&u, &mut v, &mut tmp);
        mult_AtAv(&v, &mut u, &mut tmp);
    }
    (dot(&u, &v) / dot(&v, &v)).sqrt()
}

fn mult_AtAv(v: &[f64], out: &mut [f64], tmp: &mut [f64]) {
    mult(v, tmp, A);
    mult(tmp, out, |i, j| A(j, i));
}

fn mult<F>(v: &[f64], out: &mut [f64], a: F)
           where F: Fn(usize, usize) -> f64 + Sync {
    out.par_iter_mut().enumerate().for_each(|(i, slot)| {
        *slot = v.iter().enumerate().map(|(j, x)| x / a(i, j)).sum();
    });
}

fn A(i: usize, j: usize) -> f64 {
   ((i + j) * (i + j + 1) / 2 + i + 1) as f64
}

fn dot(v: &[f64], u: &[f64]) -> f64 {
    u.iter().zip(v).map(|(x, y)| x * y).sum()
}
