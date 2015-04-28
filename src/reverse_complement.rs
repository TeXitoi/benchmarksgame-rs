// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

//extern crate libc;
// exporting needed things from libc for linux x64 (still unstable)
#[cfg(all(target_os = "linux", any(target_arch = "x86_64", target_arch = "x86")))]
mod libc {
    #![allow(non_camel_case_types)]
    #[repr(u8)]
    pub enum c_void { __variant1, __variant2 }
    pub type c_int = i32;
    #[cfg(target_arch = "x86_64")] pub type size_t = u64;
    #[cfg(target_arch = "x86")] pub type size_t = u32;
    extern { pub fn memchr(cx: *const c_void, c: c_int, n: size_t) -> *mut c_void; }
}

use std::io::{Read, Write};
use std::ptr::copy;
use std::thread;

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

/// Finds the first position at which `b` occurs in `s`.
fn memchr(h: &[u8], n: u8) -> Option<usize> {
    use libc::{c_void, c_int, size_t};
    let res = unsafe {
        libc::memchr(h.as_ptr() as *const c_void, n as c_int, h.len() as size_t)
    };
    if res.is_null() {
        None
    } else {
        Some(res as usize - h.as_ptr() as usize)
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
        let tmp = std::mem::replace(&mut self.s, &mut []);
        let tmp = match memchr(tmp, b'\n') {
            Some(i) => &mut tmp[i + 1 ..],
            None => return None,
        };
        let (seq, tmp) = match memchr(tmp, b'>') {
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

/// Length of a normal line without the terminating \n.
const LINE_LEN: usize = 60;

/// Compute the reverse complement.
fn reverse_complement(seq: &mut [u8], tables: &Tables) {
    let len = seq.len() - 1;
    let seq = &mut seq[..len];// Drop the last newline
    let off = LINE_LEN - len % (LINE_LEN + 1);
    let mut i = LINE_LEN;
    while i < len {
        unsafe {
            copy(seq.as_ptr().offset((i - off) as isize),
                 seq.as_mut_ptr().offset((i - off + 1) as isize), off);
            *seq.get_unchecked_mut(i - off) = b'\n';
        }
        i += LINE_LEN + 1;
    }

    let div = len / 4;
    let rem = len % 4;
    unsafe {
        let mut left = seq.as_mut_ptr() as *mut u16;
        // This is slow if len % 2 != 0 but still faster than bytewise operations.
        let mut right = seq.as_mut_ptr().offset(len as isize - 2) as *mut u16;
        let end = left.offset(div as isize);
        while left != end {
            let tmp = tables.cpl16(*left);
            *left = tables.cpl16(*right);
            *right = tmp;
            left = left.offset(1);
            right = right.offset(-1);
        }

        let end = end as *mut u8;
        match rem {
            1 => *end = tables.cpl8(*end),
            2 => {
                let tmp = tables.cpl8(*end);
                *end = tables.cpl8(*end.offset(1));
                *end.offset(1) = tmp;
            },
            3 => {
                *end.offset(1) = tables.cpl8(*end.offset(1));
                let tmp = tables.cpl8(*end);
                *end = tables.cpl8(*end.offset(2));
                *end.offset(2) = tmp;
            },
            _ => { },
        }
    }
}

struct Racy<T>(T);
unsafe impl<T: 'static> Send for Racy<T> {}

/// Executes a closure in parallel over the given iterator over mutable slice.
/// The closure `f` is run in parallel with an element of `iter`.
fn parallel<'a, I, T, F>(iter: I, ref f: F)
        where T: 'static + Send + Sync,
              I: Iterator<Item=&'a mut [T]>,
              F: Fn(&mut [T]) + Sync {
    let jhs = iter.map(|chunk| {
        // Need to convert `f` and `chunk` to something that can cross the task
        // boundary.
        let f = Racy(f as *const F as *const usize);
        let raw = Racy((&mut chunk[0] as *mut T, chunk.len()));
        thread::spawn(move|| {
            let f = f.0 as *const F;
            let raw = raw.0;
            unsafe { (*f)(std::slice::from_raw_parts_mut(raw.0, raw.1)) }
        })
    }).collect::<Vec<_>>();
    for jh in jhs { jh.join().unwrap(); }
}

fn main() {
    let stdin = std::io::stdin();
    let mut data = Vec::with_capacity(1024 * 1024);
    stdin.lock().read_to_end(&mut data).unwrap();
    let tables = &Tables::new();
    parallel(mut_dna_seqs(&mut data), |seq| reverse_complement(seq, tables));
    let stdout = std::io::stdout();
    stdout.lock().write_all(&data).unwrap();
}
