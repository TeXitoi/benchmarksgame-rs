// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

use std::cmp::min;
use std::env;
use std::io;
use std::io::prelude::*;

const LINE_LEN: usize = 60;
const LOOKUP_SIZE: usize = 4 * 1024;
const LOOKUP_SCALE: f32 = (LOOKUP_SIZE - 1) as f32;

// Random number generator constants
const IM: u32 = 139968;
const IA: u32 = 3877;
const IC: u32 = 29573;

const ALU: &'static str = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTG\
                            GGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGA\
                            GACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAA\
                            AATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAAT\
                            CCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAAC\
                            CCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTG\
                            CACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";

const NULL_AMINO_ACID: AminoAcid = AminoAcid { c: ' ' as u8, p: 0.0 };

static IUB: [AminoAcid;15] = [
    AminoAcid { c: 'a' as u8, p: 0.27 },
    AminoAcid { c: 'c' as u8, p: 0.12 },
    AminoAcid { c: 'g' as u8, p: 0.12 },
    AminoAcid { c: 't' as u8, p: 0.27 },
    AminoAcid { c: 'B' as u8, p: 0.02 },
    AminoAcid { c: 'D' as u8, p: 0.02 },
    AminoAcid { c: 'H' as u8, p: 0.02 },
    AminoAcid { c: 'K' as u8, p: 0.02 },
    AminoAcid { c: 'M' as u8, p: 0.02 },
    AminoAcid { c: 'N' as u8, p: 0.02 },
    AminoAcid { c: 'R' as u8, p: 0.02 },
    AminoAcid { c: 'S' as u8, p: 0.02 },
    AminoAcid { c: 'V' as u8, p: 0.02 },
    AminoAcid { c: 'W' as u8, p: 0.02 },
    AminoAcid { c: 'Y' as u8, p: 0.02 },
];

static HOMO_SAPIENS: [AminoAcid;4] = [
    AminoAcid { c: 'a' as u8, p: 0.3029549426680 },
    AminoAcid { c: 'c' as u8, p: 0.1979883004921 },
    AminoAcid { c: 'g' as u8, p: 0.1975473066391 },
    AminoAcid { c: 't' as u8, p: 0.3015094502008 },
];

fn sum_and_scale(a: &'static [AminoAcid]) -> Vec<AminoAcid> {
    let mut p = 0f32;
    let mut result: Vec<AminoAcid> = a.iter().map(|a_i| {
        p += a_i.p; 
        AminoAcid { c: a_i.c, p: p * LOOKUP_SCALE }
    }).collect();
    let result_len = result.len();
    result[result_len - 1].p = LOOKUP_SCALE;
    result
}

#[derive(Copy, Clone)]
struct AminoAcid {
    c: u8,
    p: f32,
}

struct RepeatFasta<'a, W:'a> {
    alu: &'static str,
    out: &'a mut W
}

impl<'a, W: Write> RepeatFasta<'a, W> {
    fn new(alu: &'static str, w: &'a mut W) -> RepeatFasta<'a, W> {
        RepeatFasta { alu: alu, out: w }
    }

    fn make(&mut self, n: usize) -> io::Result<()> {
        let alu_len = self.alu.len();
        let mut buf = vec![0; alu_len + LINE_LEN];
        let alu: &[u8] = self.alu.as_bytes();

        for (slot, val) in buf.iter_mut().zip(alu) {
            *slot = *val;
        }
        let buf_len = buf.len();
        for (slot, val) in buf[alu_len..buf_len].iter_mut().zip(&alu[..LINE_LEN]) {
            *slot = *val;
        }

        let mut pos = 0;
        let mut bytes;
        let mut n = n;
        while n > 0 {
            bytes = min(LINE_LEN, n);
            try!(self.out.write_all(&buf[pos..pos + bytes]));
            try!(self.out.write_all(&[b'\n']));
            pos += bytes;
            if pos > alu_len {
                pos -= alu_len;
            }
            n -= bytes;
        }
        Ok(())
    }
}

fn make_lookup(a: &[AminoAcid]) -> [AminoAcid;LOOKUP_SIZE] {
    let mut lookup = [ NULL_AMINO_ACID;LOOKUP_SIZE ];
    let mut j = 0;
    for (i, slot) in lookup.iter_mut().enumerate() {
        while a[j].p < (i as f32) {
            j += 1;
        }
        *slot = a[j];
    }
    lookup
}

struct RandomFasta<'a, W:'a> {
    seed: u32,
    lookup: [AminoAcid;LOOKUP_SIZE],
    out: &'a mut W,
}

impl<'a, W: Write> RandomFasta<'a, W> {
    fn new(w: &'a mut W, a: &[AminoAcid]) -> RandomFasta<'a, W> {
        RandomFasta {
            seed: 42,
            out: w,
            lookup: make_lookup(a),
        }
    }

    fn rng(&mut self, max: f32) -> f32 {
        self.seed = (self.seed * IA + IC) % IM;
        (max * self.seed as f32) / (IM as f32)
    }

    fn nextc(&mut self) -> u8 {
        let r = self.rng(LOOKUP_SCALE);
        for i in (r as usize..LOOKUP_SIZE) {
            if self.lookup[i].p >= r { 
                return self.lookup[i].c;
            }
        }
        unreachable!();
    }

    fn make(&mut self, n: usize) -> io::Result<()> {
        let lines = n / LINE_LEN;
        let chars_left = n % LINE_LEN;
        let mut buf = [0;LINE_LEN + 1];

        for _ in 0..lines {
            for i in 0..LINE_LEN {
                buf[i] = self.nextc();
            }
            buf[LINE_LEN] = '\n' as u8;
            try!(self.out.write(&buf));
        }
        for i in 0..chars_left {
            buf[i] = self.nextc();
        }
        self.out.write_all(&buf[..chars_left])
    }
}

fn main() {
    let mut args = env::args();
    let n = if args.len() > 1 {
        args.nth(1).unwrap().parse::<usize>().unwrap()
    } else {
        5
    };
    
    let stdout = io::stdout();
    let mut out = stdout.lock();

    out.write_all(b">ONE Homo sapiens alu\n").unwrap();
    {
        let mut repeat = RepeatFasta::new(ALU, &mut out);
        repeat.make(n * 2).unwrap();
    }

    out.write_all(b">TWO IUB ambiguity codes\n").unwrap();
    let iub = sum_and_scale(&IUB);
    let mut random = RandomFasta::new(&mut out, &iub);
    random.make(n * 3).unwrap();

    random.out.write_all(b">THREE Homo sapiens frequency\n").unwrap();
    let homo_sapiens = sum_and_scale(&HOMO_SAPIENS);
    random.lookup = make_lookup(&homo_sapiens);
    random.make(n * 5).unwrap();

    random.out.write_all(b"\n").unwrap();
}
