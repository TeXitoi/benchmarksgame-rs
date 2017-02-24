// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by Matt Brubeck
// contributed by TeXitoi

#![allow(non_snake_case)]

use std::thread;

// As std::simd::f64x2 etc. are unstable, we provide a similar interface,
// expecting llvm to autovectorize its usage.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
struct usizex2(usize, usize);
impl std::ops::Add for usizex2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        usizex2(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl std::ops::Mul for usizex2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        usizex2(self.0 * rhs.0, self.1 * rhs.1)
    }
}
impl std::ops::Div for usizex2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        usizex2(self.0 / rhs.0, self.1 / rhs.1)
    }
}
impl From<usizex2> for f64x2 {
    fn from(i: usizex2) -> f64x2 {
        f64x2(i.0 as f64, i.1 as f64)
    }
}

#[allow(non_camel_case_types)]
struct f64x2(f64, f64);
impl std::ops::Add for f64x2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        f64x2(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl std::ops::Div for f64x2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        f64x2(self.0 / rhs.0, self.1 / rhs.1)
    }
}

fn main() {
    let n = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok())
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
    mult_Av(v, tmp);
    mult_Atv(tmp, out);
}

fn mult_Av(v: &[f64], out: &mut [f64]) {
    parallel(out, |start, out| mult(v, out, start, Ax2));
}

fn mult_Atv(v: &[f64], out: &mut [f64]) {
    parallel(out, |start, out| mult(v, out, start, |i, j| Ax2(j, i)));
}

fn mult<F>(v: &[f64], out: &mut [f64], start: usize, a: F)
           where F: Fn(usizex2, usizex2) -> f64x2 {
    for (i, slot) in out.iter_mut().enumerate().map(|(i, s)| (i + start, s)) {
        let mut sum = f64x2(0.0, 0.0);
        for (j, chunk) in v.chunks(2).enumerate().map(|(j, s)| (2 * j, s)) {
            let top = f64x2(chunk[0], chunk[1]);
            let bot = a(usizex2(i, i), usizex2(j, j+1));
            sum = sum + top / bot;
        }
        let f64x2(a, b) = sum;
        *slot = a + b;
    }
}

fn Ax2(i: usizex2, j: usizex2) -> f64x2 {
    ((i + j) * (i + j + usizex2(1, 1)) / usizex2(2, 2) + i + usizex2(1, 1)).into()
}

fn dot(v: &[f64], u: &[f64]) -> f64 {
    v.iter().zip(u.iter()).map(|(a, b)| *a * *b).fold(0., |acc, i| acc + i)
}

struct Racy<T>(T);
unsafe impl<T: 'static> Send for Racy<T> {}

// Executes a closure in parallel over the given mutable slice. The closure `f`
// is run in parallel and yielded the starting index within `v` as well as a
// sub-slice of `v`.
fn parallel<'a, T, F>(v: &mut [T], ref f: F)
    where T: 'static + Send + Sync,
          F: Fn(usize, &mut [T]) + Sync {
    let size = v.len() / 4 + 1;
    let jhs = v.chunks_mut(size).enumerate().map(|(i, chunk)| {
        // Need to convert `f` and `chunk` to something that can cross the task
        // boundary.
        let f = Racy(f as *const F as *const usize);
        let raw = Racy((&mut chunk[0] as *mut T, chunk.len()));
        thread::spawn(move|| {
            let f = f.0 as *const F;
            let raw = raw.0;
            unsafe { (*f)(i * size, std::slice::from_raw_parts_mut(raw.0, raw.1)) }
        })
    }).collect::<Vec<_>>();
    for jh in jhs { jh.join().unwrap(); }
}
