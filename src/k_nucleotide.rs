// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

extern crate fnv;

use std::sync::Arc;
use std::thread;

type Map = fnv::FnvHashMap<Code, u32>;

static TABLE: [u8;4] = [ 'A' as u8, 'C' as u8, 'G' as u8, 'T' as u8 ];

static OCCURRENCES: [&'static str;5] = [
    "GGT",
    "GGTA",
    "GGTATT",
    "GGTATTTTAATT",
    "GGTATTTTAATTTATAGT",
];

// Code implementation

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Code(u64);

impl Code {
    fn hash(&self) -> u64 {
        let Code(ret) = *self;
        return ret;
    }

    fn push_char(&self, c: u8) -> Code {
        Code((self.hash() << 2) + (pack_symbol(c) as u64))
    }

    fn rotate(&self, c: u8, frame: usize) -> Code {
        Code(self.push_char(c).hash() & ((1u64 << (2 * frame)) - 1))
    }

    fn pack(string: &str) -> Code {
        string.bytes().fold(Code(0u64), |a, b| a.push_char(b))
    }

    fn unpack(&self, frame: usize) -> String {
        let mut key = self.hash();
        let mut result = Vec::new();
        for _ in 0..frame {
            result.push(unpack_symbol((key as u8) & 3));
            key >>= 2;
        }

        result.reverse();
        String::from_utf8(result).unwrap()
    }
}

fn pack_symbol(c: u8) -> u8 {
    match c as char {
        'A' => 0,
        'C' => 1,
        'G' => 2,
        'T' => 3,
        _ => panic!("{}", c as char),
    }
}

fn unpack_symbol(c: u8) -> u8 {
    TABLE[c as usize]
}

fn generate_frequencies(mut input: &[u8], frame: usize) -> Map {
    let mut frequencies = Map::default();
    if input.len() < frame { return frequencies; }
    let mut code = Code(0);

    // Pull first frame.
    for _ in 0..frame {
        code = code.push_char(input[0]);
        input = &input[1..];
    }
    *frequencies.entry(code).or_insert(0) += 1;

    while input.len() != 0 && input[0] != ('>' as u8) {
        code = code.rotate(input[0], frame);
        *frequencies.entry(code).or_insert(0) += 1;
        input = &input[1..];
    }
    frequencies
}

fn print_frequencies(frequencies: &Map, frame: usize) {
    let mut vector = Vec::new();
    for (&code, &count) in frequencies.iter() {
        vector.push((count, code));
    }
    vector.sort();

    let mut total_count = 0;
    for &(count, _) in vector.iter() {
        total_count += count;
    }

    for &(count, key) in vector.iter().rev() {
        println!("{} {:.3}",
                 key.unpack(frame),
                 (count as f32 * 100.0) / (total_count as f32));
    }
    println!("");
}

fn print_occurrences(frequencies: &Map, occurrence: &'static str) {
    println!("{}\t{}", frequencies[&Code::pack(occurrence)], occurrence);
}

fn get_sequence<R: std::io::BufRead>(r: R, key: &str) -> Vec<u8> {
    let mut res = Vec::new();
    for l in r.lines().map(|l| l.ok().unwrap())
        .skip_while(|l| key != &l[..key.len()]).skip(1)
    {
        use std::ascii::AsciiExt;
        res.extend(l.trim().as_bytes().iter().map(|b| b.to_ascii_uppercase()));
    }
    res
}

fn main() {
    let stdin = std::io::stdin();
    let input = get_sequence(stdin.lock(), ">THREE");
    let input = Arc::new(input);

    let occ_freqs: Vec<_> = OCCURRENCES.iter().map(|&occ| {
        let input = input.clone();
        thread::spawn(move|| generate_frequencies(&input, occ.len()))
    }).collect();

    for i in 1..3 {
        print_frequencies(&generate_frequencies(&input, i), i);
    }
    for (&occ, freq) in OCCURRENCES.iter().zip(occ_freqs.into_iter()) {
        print_occurrences(&mut freq.join().unwrap(), occ);
    }
}
