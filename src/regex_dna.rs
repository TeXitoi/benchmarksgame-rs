// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by BurntSushi
// contributed by TeXitoi

extern crate regex;

use std::io::{self, Read};
use std::sync::Arc;
use std::thread;
use regex::NoExpand;

macro_rules! regex { ($re:expr) => (::regex::Regex::new($re).unwrap()); }

fn main() {
    let mut seq = String::new();
    io::stdin().read_to_string(&mut seq).unwrap();
    let ilen = seq.len();

    seq = regex!(">[^\n]*\n|\n").replace_all(&seq, NoExpand(""));
    let seq_arc = Arc::new(seq.clone()); // copy before it moves
    let clen = seq.len();

    let seqlen = thread::spawn(move|| {
        let substs = vec![
            (regex!("B"), "(c|g|t)"),
            (regex!("D"), "(a|g|t)"),
            (regex!("H"), "(a|c|t)"),
            (regex!("K"), "(g|t)"),
            (regex!("M"), "(a|c)"),
            (regex!("N"), "(a|c|g|t)"),
            (regex!("R"), "(a|g)"),
            (regex!("S"), "(c|g)"),
            (regex!("V"), "(a|c|g)"),
            (regex!("W"), "(a|t)"),
            (regex!("Y"), "(c|t)"),
        ];
        let mut seq = seq;
        for (re, replacement) in substs.into_iter() {
            seq = re.replace_all(&seq, NoExpand(replacement));
        }
        seq.len()
    });

    let variants = vec![
        regex!("agggtaaa|tttaccct"),
        regex!("[cgt]gggtaaa|tttaccc[acg]"),
        regex!("a[act]ggtaaa|tttacc[agt]t"),
        regex!("ag[act]gtaaa|tttac[agt]ct"),
        regex!("agg[act]taaa|ttta[agt]cct"),
        regex!("aggg[acg]aaa|ttt[cgt]ccct"),
        regex!("agggt[cgt]aa|tt[acg]accct"),
        regex!("agggta[cgt]a|t[acg]taccct"),
        regex!("agggtaa[cgt]|[acg]ttaccct"),
    ];
    let (mut variant_strs, mut counts) = (vec!(), vec!());
    for variant in variants.into_iter() {
        let seq_arc_copy = seq_arc.clone();
        variant_strs.push(variant.to_string());
        counts.push(thread::spawn(move|| {
            variant.find_iter(&seq_arc_copy).count()
        }));
    }

    let mut olines = Vec::new();
    for (variant, count) in variant_strs.iter().zip(counts.into_iter()) {
        olines.push(format!("{} {}", variant, count.join().unwrap()));
    }
    olines.push("".to_string());
    olines.push(format!("{}", ilen));
    olines.push(format!("{}", clen));
    olines.push(format!("{}", seqlen.join().unwrap()));
    println!("{}", olines.connect("\n"));
}
