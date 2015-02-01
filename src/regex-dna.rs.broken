// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

extern crate regex;

use std::io;
use regex::{NoExpand, Regex};
use std::sync::{Arc, Future};

macro_rules! regex {
    ($e:expr) => (Regex::new($e).unwrap())
}

fn count_matches(seq: &str, variant: &Regex) -> i32 {
    let mut n = 0;
    for _ in variant.find_iter(seq) {
        n += 1;
    }
    n
}

fn main() {
    let mut rdr = io::stdin();
    let mut seq = rdr.read_to_string().unwrap();
    let ilen = seq.len();

    seq = regex!(">[^\n]*\n|\n").replace_all(&*seq, NoExpand(""));
    let seq_arc = Arc::new(seq.clone()); // copy before it moves
    let clen = seq.len();

    let mut seqlen = Future::spawn(move|| {
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
            seq = re.replace_all(&*seq, NoExpand(replacement));
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
        counts.push(Future::spawn(move|| {
            count_matches(&**seq_arc_copy, &variant)
        }));
    }

    for (i, variant) in variant_strs.iter().enumerate() {
        println!("{} {}", variant, counts[i].get());
    }
    println!("");
    println!("{}", ilen);
    println!("{}", clen);
    println!("{}", seqlen.get());
}
