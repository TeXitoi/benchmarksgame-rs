// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

extern crate futures;
extern crate futures_cpupool;

use std::sync::Arc;
use std::hash::{Hasher, BuildHasherDefault};
use std::collections::HashMap;
use futures::Future;
use futures_cpupool::CpuPool;
use Item::*;

struct NaiveHasher(u64);
impl Default for NaiveHasher {
    fn default() -> Self {
        NaiveHasher(0)
    }
}
impl Hasher for NaiveHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unimplemented!()
    }
    fn write_u64(&mut self, i: u64) {
        self.0 = i ^ i >> 7;
    }
}
type NaiveBuildHasher = BuildHasherDefault<NaiveHasher>;
type NaiveHashMap<K, V> = HashMap<K, V, NaiveBuildHasher>;
type Map = NaiveHashMap<Code, u32>;

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Code(u64);
impl Code {
    fn push(&mut self, c: u8, mask: u64) {
        self.0 <<= 2;
        self.0 |= c as u64;
        self.0 &= mask;
    }
    fn from_str(s: &str) -> Code {
        let mask = Code::make_mask(s.len());
        let mut res = Code(0);
        for c in s.as_bytes() {
            res.push(Code::encode(*c), mask);
        }
        res
    }
    fn to_string(&self, frame: usize) -> String {
        let mut res = vec![];
        let mut code = self.0;
        for _ in 0..frame {
            let c = match code as u8 & 0b11 {
                c if c == Code::encode(b'A') => b'A',
                c if c == Code::encode(b'T') => b'T',
                c if c == Code::encode(b'G') => b'G',
                c if c == Code::encode(b'C') => b'C',
                _ => unreachable!(),
            };
            res.push(c);
            code >>= 2;
        }
        res.reverse();
        String::from_utf8(res).unwrap()
    }
    fn make_mask(frame: usize) -> u64 {
        (1u64 << (2 * frame)) - 1
    }
    fn encode(c: u8) -> u8 {
        (c & 0b110) >> 1
    }
}

struct Iter<'a> {
    iter: std::slice::Iter<'a, u8>,
    code: Code,
    mask: u64,
}
impl<'a> Iter<'a> {
    fn new(input: &[u8], frame: usize) -> Iter {
        let mut iter = input.iter();
        let mut code = Code(0);
        let mask = Code::make_mask(frame);
        for c in iter.by_ref().take(frame - 1) {
            code.push(*c, mask);
        }
        Iter {
            iter: iter,
            code: code,
            mask: mask,
        }
    }
}
impl<'a> Iterator for Iter<'a> {
    type Item = Code;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&c| {
            self.code.push(c, self.mask);
            self.code
        })
    }
}

fn gen_freq(input: &[u8], frame: usize) -> Map {
    let mut freq = Map::default();
    for code in Iter::new(input, frame) {
        *freq.entry(code).or_insert(0) += 1;
    }
    freq
}

#[derive(Clone, Copy)]
enum Item {
    Freq(usize),
    Occ(&'static str),
}
impl Item {
    fn print(&self, freq: &Map) {
        match *self {
            Freq(frame) => {
                let mut v: Vec<_> = freq.iter().map(|(&code, &count)| (count, code)).collect();
                v.sort();
                let total = v.iter().map(|&(count, _)| count).sum::<u32>() as f32;
                for &(count, key) in v.iter().rev() {
                    println!("{} {:.3}", key.to_string(frame), (count as f32 * 100.) / total);
                }
                println!("");
            }
            Occ(occ) => println!("{}\t{}", freq[&Code::from_str(occ)], occ),
        }
    }
    fn gen_freq(&self, input: &[u8]) -> Map {
        match *self {
            Freq(frame) => gen_freq(input, frame),
            Occ(occ) => gen_freq(input, occ.len()),
        }
    }
}
static ITEMS: [Item; 7] = [
    Freq(1),
    Freq(2),
    Occ("GGT"),
    Occ("GGTA"),
    Occ("GGTATT"),
    Occ("GGTATTTTAATT"),
    Occ("GGTATTTTAATTTATAGT"),
];


fn get_seq<R: std::io::BufRead>(r: R, key: &str) -> Vec<u8> {
    let mut res = Vec::new();
    for l in r.lines().map(|l| l.unwrap()).skip_while(|l| !l.starts_with(key)).skip(1) {
        res.extend(l.trim().as_bytes().iter().cloned().map(Code::encode));
    }
    res
}

fn main() {
    let stdin = std::io::stdin();
    let input = get_seq(stdin.lock(), ">THREE");
    let input = Arc::new(input);
    let pool = CpuPool::new_num_cpus();

    // In reverse to spawn big tasks first
    let items: Vec<_> = ITEMS.iter().rev().map(|&item| {
        let input = input.clone();
        let future_freq = pool.spawn_fn(move || Ok::<_, ()>(item.gen_freq(&input)));
        (item, future_freq)
    }).collect();

    for (item, future_freq) in items.into_iter().rev() {
        item.print(&future_freq.wait().unwrap());
    }
}
