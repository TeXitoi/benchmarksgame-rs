// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by BurntSushi

extern crate regex;

use regex::bytes::Regex;

use std::io::{self, Read};
use std::sync::Arc;
use std::thread;

macro_rules! regex { ($re:expr) => { Regex::new($re).unwrap() } }

fn main() {
    let mut seq = Vec::with_capacity(50 * (1 << 20));
    io::stdin().read_to_end(&mut seq).unwrap();
    let ilen = seq.len();

    seq = regex!(r">[^\n]*\n|\n").replace_all(&seq, &b""[..]);
    let clen = seq.len();
    let seq_arc = Arc::new(seq.clone());

    let variants = vec![
        regex!(r"agggtaaa|tttaccct"),
        regex!(r"[cgt]gggtaaa|tttaccc[acg]"),
        regex!(r"a[act]ggtaaa|tttacc[agt]t"),
        regex!(r"ag[act]gtaaa|tttac[agt]ct"),
        regex!(r"agg[act]taaa|ttta[agt]cct"),
        regex!(r"aggg[acg]aaa|ttt[cgt]ccct"),
        regex!(r"agggt[cgt]aa|tt[acg]accct"),
        regex!(r"agggta[cgt]a|t[acg]taccct"),
        regex!(r"agggtaa[cgt]|[acg]ttaccct"),
    ];
    let mut counts = vec![];
    for variant in variants {
        let seq = seq_arc.clone();
        let restr = variant.to_string();
        let future = thread::spawn(move || variant.find_iter(&seq).count());
        counts.push((restr, future));
    }

    let substs = vec![
        (regex!(r"B"), &b"(c|g|t)"[..]),
        (regex!(r"D"), &b"(a|g|t)"[..]),
        (regex!(r"H"), &b"(a|c|t)"[..]),
        (regex!(r"K"), &b"(g|t)"[..]),
        (regex!(r"M"), &b"(a|c)"[..]),
        (regex!(r"N"), &b"(a|c|g|t)"[..]),
        (regex!(r"R"), &b"(a|g)"[..]),
        (regex!(r"S"), &b"(c|g)"[..]),
        (regex!(r"V"), &b"(a|c|g)"[..]),
        (regex!(r"W"), &b"(a|t)"[..]),
        (regex!(r"Y"), &b"(c|t)"[..]),
    ];
    let mut seq = seq;
    for (re, replacement) in substs.into_iter() {
        seq = re.replace_all(&seq, replacement);
    }
    let rlen = seq.len();

    for (variant, count) in counts {
        println!("{} {}", variant, count.join().unwrap());
    }
    println!("\n{}\n{}\n{}", ilen, clen, rlen);
}
