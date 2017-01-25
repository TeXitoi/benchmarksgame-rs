// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

extern crate fnv;

use std::sync::Arc;
use std::thread;

type Map = fnv::FnvHashMap<Code, u32>;

static OCCURRENCES: [&'static str; 5] = [
    "GGT",
    "GGTA",
    "GGTATT",
    "GGTATTTTAATT",
    "GGTATTTTAATTTATAGT",
];

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Code(u64);

impl Code {
    fn hash(&self) -> u64 {
        let Code(ret) = *self;
        return ret;
    }

    fn push_char(&self, c: u8) -> Code {
        Code((self.hash() << 2) | (c as u64))
    }

    fn rotate(&self, c: u8, mask: u64) -> Code {
        Code(self.push_char(c).hash() & mask)
    }

    fn pack(string: &str) -> Code {
        string.bytes().fold(Code(0u64), |a, b| a.push_char(pack_symbol(b)))
    }

    fn unpack(&self, frame: usize) -> String {
        let mut key = self.hash();
        let mut result = Vec::new();
        for _ in 0..frame {
            result.push(unpack_symbol((key as u8) & 0b11));
            key >>= 2;
        }

        result.reverse();
        String::from_utf8(result).unwrap()
    }
}

fn make_mask(frame: usize) -> u64 {
    (1u64 << (2 * frame)) - 1
}

fn pack_symbol(c: u8) -> u8 {
    (c & 0b110) >> 1
}

fn unpack_symbol(c: u8) -> u8 {
    match c {
        c if c == pack_symbol(b'A') => b'A',
        c if c == pack_symbol(b'T') => b'T',
        c if c == pack_symbol(b'G') => b'G',
        c if c == pack_symbol(b'C') => b'C',
        _ => unreachable!(),
    }
}

fn generate_frequencies(input: &[u8], frame: usize) -> Map {
    let mut frequencies = Map::default();
    if input.len() < frame { return frequencies; }
    let mut code = Code(0);
    let mut iter = input.iter().cloned();

    for c in iter.by_ref().take(frame - 1) {
        code = code.push_char(c);
    }

    let mask = make_mask(frame);
    for c in iter {
        code = code.rotate(c, mask);
        *frequencies.entry(code).or_insert(0) += 1;
    }
    frequencies
}

fn print_frequencies(frequencies: &Map, frame: usize) {
    let mut vector: Vec<_> = frequencies.iter().map(|(&code, &count)| (count, code)).collect();
    vector.sort();
    let total_count = vector.iter().map(|&(count, _)| count).sum::<u32>() as f32;

    for &(count, key) in vector.iter().rev() {
        println!("{} {:.3}", key.unpack(frame), (count as f32 * 100.0) / total_count);
    }
    println!("");
}

fn print_occurrences(frequencies: &Map, occurrence: &'static str) {
    println!("{}\t{}", frequencies[&Code::pack(occurrence)], occurrence);
}

fn get_sequence<R: std::io::BufRead>(r: R, key: &str) -> Vec<u8> {
    let mut res = Vec::new();
    for l in r.lines().map(|l| l.unwrap()).skip_while(|l| !l.starts_with(key)).skip(1) {
        res.extend(l.trim().as_bytes().iter().cloned().map(pack_symbol));
    }
    res
}

fn main() {
    let stdin = std::io::stdin();
    let input = get_sequence(stdin.lock(), ">THREE");
    let input = Arc::new(input);

    let occ_freqs: Vec<_> = OCCURRENCES.iter().skip(2).map(|&occ| {
        let input = input.clone();
        thread::spawn(move|| generate_frequencies(&input, occ.len()))
    }).collect();

    for i in 1..3 {
        print_frequencies(&generate_frequencies(&input, i), i);
    }
    for &occ in OCCURRENCES.iter().take(2) {
        print_occurrences(&generate_frequencies(&input, occ.len()), occ);
    }
    for (&occ, freq) in OCCURRENCES.iter().skip(2).zip(occ_freqs.into_iter()) {
        print_occurrences(&freq.join().unwrap(), occ);
    }
}
