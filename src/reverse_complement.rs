// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by Matt Brubeck
// contributed by TeXitoi

extern crate crossbeam;
extern crate num_cpus;
extern crate memchr;

use std::io::{Read, Write};
use std::{cmp, io, mem, ptr, slice};
use std::fs::File;

struct Tables {
    table8: [u8;1 << 8],
    table16: [u16;1 << 16]
}

impl Tables {
    fn new() -> Tables {
        let mut table8 = [0;1 << 8];
        for (i, v) in table8.iter_mut().enumerate() {
            *v = Tables::computed_cpl8(i as u8);
        }
        let mut table16 = [0;1 << 16];
        for (i, v) in table16.iter_mut().enumerate() {
            *v = (table8[i & 255] as u16) << 8 |
                 table8[i >> 8]  as u16;
        }
        Tables { table8: table8, table16: table16 }
    }

    fn computed_cpl8(c: u8) -> u8 {
        match c {
            b'A' | b'a' => b'T',
            b'C' | b'c' => b'G',
            b'G' | b'g' => b'C',
            b'T' | b't' => b'A',
            b'U' | b'u' => b'A',
            b'M' | b'm' => b'K',
            b'R' | b'r' => b'Y',
            b'W' | b'w' => b'W',
            b'S' | b's' => b'S',
            b'Y' | b'y' => b'R',
            b'K' | b'k' => b'M',
            b'V' | b'v' => b'B',
            b'H' | b'h' => b'D',
            b'D' | b'd' => b'H',
            b'B' | b'b' => b'V',
            b'N' | b'n' => b'N',
            i => i,
        }
    }

    /// Retrieves the complement for `i`.
    fn cpl8(&self, i: u8) -> u8 {
        self.table8[i as usize]
    }

    /// Retrieves the complement for `i`.
    fn cpl16(&self, i: u16) -> u16 {
        self.table16[i as usize]
    }
}

/// A mutable iterator over DNA sequences
struct MutDnaSeqs<'a> { s: &'a mut [u8] }
fn mut_dna_seqs<'a>(s: &'a mut [u8]) -> MutDnaSeqs<'a> {
    MutDnaSeqs { s: s }
}
impl<'a> Iterator for MutDnaSeqs<'a> {
    type Item = &'a mut [u8];

    fn next(&mut self) -> Option<&'a mut [u8]> {
        let tmp = mem::replace(&mut self.s, &mut []);
        let tmp = match memchr::memchr(b'\n', tmp) {
            Some(i) => &mut tmp[i + 1 ..],
            None => return None,
        };
        let (seq, tmp) = match memchr::memchr(b'>', tmp) {
            Some(i) => tmp.split_at_mut(i),
            None => {
                let len = tmp.len();
                tmp.split_at_mut(len)
            }
        };
        self.s = tmp;
        Some(seq)
    }
}

/// An iterator that yields chunks from the front of one slice and the back of the other.
struct DoubleChunk<'a, T: 'a> {
    n: usize,
    left: &'a mut [T],
    right: &'a mut [T],
}
fn double_chunk<'a, T>(n: usize, left: &'a mut [T], right: &'a mut [T]) -> DoubleChunk<'a, T> {
    DoubleChunk { n: n, left: left, right: right }
}
impl<'a, T> Iterator for DoubleChunk<'a, T> {
    type Item = (&'a mut [T], &'a mut [T]);

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.left.len();
        if len == 0 {
            return None;
        }
        let n = cmp::min(self.n, len);
        let (x, left)  = mem::replace(&mut self.left,  &mut []).split_at_mut(n);
        let (right, y) = mem::replace(&mut self.right, &mut []).split_at_mut(len - n);
        self.left = left;
        self.right = right;
        Some((x, y))
    }
}

/// Length of a normal line without the terminating \n.
const LINE_LEN: usize = 60;

/// Compute the reverse complement.
fn reverse_complement(seq: &mut [u8], tables: &Tables) {
    let len = seq.len() - 1;
    let seq = &mut seq[..len];// Drop the last newline

    // Move newlines so the reversed text is wrapped correctly.
    let off = LINE_LEN - len % (LINE_LEN + 1);
    let mut i = LINE_LEN;
    while i < len {
        unsafe {
            ptr::copy(seq.as_ptr().offset((i - off) as isize),
                      seq.as_mut_ptr().offset((i - off + 1) as isize), off);
            *seq.get_unchecked_mut(i - off) = b'\n';
        }
        i += LINE_LEN + 1;
    }

    let div = len / 4;
    let rem = len % 4;

    let thread_count = num_cpus::get();
    let chunk_size = (div + thread_count - 1) / thread_count;
    crossbeam::scope(|scope| {
        let (left, right);
        unsafe {
            let p = seq.as_mut_ptr();
            left = slice::from_raw_parts_mut(p as *mut u16, div);
            // This is slow if len % 2 != 0 but still faster than bytewise operations.
            let q = p.offset((div * 2 + rem) as isize);
            right = slice::from_raw_parts_mut(q as *mut u16, div);
        };
        for (left, right) in double_chunk(chunk_size, left, right) {
            scope.spawn(move || {
                for (left, right) in left.iter_mut().zip(right.iter_mut().rev()) {
                    let tmp = tables.cpl16(*left);
                    *left = tables.cpl16(*right);
                    *right = tmp;
                }
            });
        }
    });

    match rem {
        0 => {}
        1 => seq[div * 2] = tables.cpl8(seq[div * 2]),
        2 => {
            let tmp = tables.cpl8(seq[div * 2]);
            seq[div * 2] = tables.cpl8(seq[div * 2 + 1]);
            seq[div * 2 + 1] = tmp;
        },
        3 => {
            seq[div * 2 + 1] = tables.cpl8(seq[div * 2 + 1]);
            let tmp = tables.cpl8(seq[div * 2]);
            seq[div * 2] = tables.cpl8(seq[div * 2 + 2]);
            seq[div * 2 + 2] = tmp;
        },
        _ => unreachable!()
    }
}

fn file_size(f: &mut File) -> io::Result<usize> {
    Ok(f.metadata()?.len() as usize)
}

fn main() {
    let mut stdin = File::open("/dev/stdin").expect("Could not open /dev/stdin");
    let size = file_size(&mut stdin).unwrap_or(1024 * 1024);
    let mut data = Vec::with_capacity(size + 1);
    stdin.read_to_end(&mut data).unwrap();
    let tables = &Tables::new();
    crossbeam::scope(|scope| for seq in mut_dna_seqs(&mut data) {
        scope.spawn(move || reverse_complement(seq, tables));
    });
    let stdout = io::stdout();
    stdout.lock().write_all(&data).unwrap();
}
