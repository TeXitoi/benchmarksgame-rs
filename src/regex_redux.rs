// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// regex-dna program contributed by the Rust Project Developers
// contributed by BurntSushi
// contributed by TeXitoi
// converted from regex-dna program
// contributed by Matt Brubeck

extern crate regex;

use regex::bytes::Regex;
use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<Error>> {
    let mut seq = fs::read("/dev/stdin")?;
    let ilen = seq.len();

    // Remove headers and newlines.
    seq = Regex::new(">[^\n]*\n|\n")?.replace_all(&seq, &b""[..]).into_owned();
    let clen = seq.len();

    // Search for occurrences of the following patterns:
    let variants = vec![
        Regex::new("agggtaaa|tttaccct")?,
        Regex::new("[cgt]gggtaaa|tttaccc[acg]")?,
        Regex::new("a[act]ggtaaa|tttacc[agt]t")?,
        Regex::new("ag[act]gtaaa|tttac[agt]ct")?,
        Regex::new("agg[act]taaa|ttta[agt]cct")?,
        Regex::new("aggg[acg]aaa|ttt[cgt]ccct")?,
        Regex::new("agggt[cgt]aa|tt[acg]accct")?,
        Regex::new("agggta[cgt]a|t[acg]taccct")?,
        Regex::new("agggtaa[cgt]|[acg]ttaccct")?,
    ];

    // Count each pattern in parallel.  Use an Arc (atomic reference-counted
    // pointer) to share the sequence between threads without copying it.
    let seq_arc = Arc::new(seq);
    let mut counts = vec![];
    for variant in variants {
        let seq = seq_arc.clone();
        let restr = variant.to_string();
        let future = thread::spawn(move || variant.find_iter(&seq).count());
        counts.push((restr, future));
    }

    // Replace the following patterns, one at a time:
    let substs = vec![
        (Regex::new("tHa[Nt]")?, &b"<4>"[..]),
        (Regex::new("aND|caN|Ha[DS]|WaS")?, &b"<3>"[..]),
        (Regex::new("a[NSt]|BY")?, &b"<2>"[..]),
        (Regex::new("<[^>]*>")?, &b"|"[..]),
        (Regex::new("\\|[^|][^|]*\\|")?, &b"-"[..]),
    ];

    // Use Cow here to avoid one extra copy of the sequence, by borrowing from
    // the Arc during the first iteration.
    let mut seq = Cow::Borrowed(&seq_arc[..]);

    // Perform the replacements in sequence:
    for (re, replacement) in substs {
        seq = Cow::Owned(re.replace_all(&seq, replacement).into_owned());
    }

    // Print the results:
    for (variant, count) in counts {
        println!("{} {}", variant, count.join().unwrap());
    }
    println!("\n{}\n{}\n{}", ilen, clen, seq.len());
    Ok(())
}
