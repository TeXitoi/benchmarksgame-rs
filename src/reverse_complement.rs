// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by Matt Brubeck
// contributed by Cristi Cobzarenco (@cristicbz)
// contributed by TeXitoi

extern crate rayon;
extern crate memchr;

use std::io::{Read, Write};
use std::{io, ptr, slice};
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

/// Length of a normal line without the terminating \n.
const LINE_LEN: usize = 60;
const SEQUENTIAL_SIZE: usize = 1024;

/// Compute the reverse complement with the sequence split into two equal-sized slices.
fn reverse_complement_left_right(left: &mut [u16], right: &mut [u16], tables: &Tables) {
    let len = left.len();
    if len <= SEQUENTIAL_SIZE {
        assert_eq!(right.len(), len);
        for (left, right) in left.iter_mut().zip(right.iter_mut().rev()) {
            let tmp = tables.cpl16(*left);
            *left = tables.cpl16(*right);
            *right = tmp;
        }
    } else {
        let (left1, left2) = left.split_at_mut((len + 1) / 2);
        let (right2, right1) = right.split_at_mut(len / 2);
        rayon::join(|| reverse_complement_left_right(left1, right1, tables),
                    || reverse_complement_left_right(left2, right2, tables));
    }
}

/// Split a byte slice into two u16-halves with any remainder left in the middle.
fn split_mut_middle_as_u16<'a>(seq: &'a mut [u8]) -> (&'a mut [u16], &'a mut [u8], &'a mut [u16]) {
    let len = seq.len();
    let div = len / 4;
    let rem = len % 4;
    unsafe {
        let left_ptr = seq.as_mut_ptr();
        // This is slow if len % 2 != 0 but still faster than bytewise operations.
        let right_ptr = left_ptr.offset((div * 2 + rem) as isize);
        (slice::from_raw_parts_mut(left_ptr as *mut u16, div),
        slice::from_raw_parts_mut(left_ptr.offset((div * 2) as isize), rem),
        slice::from_raw_parts_mut(right_ptr as *mut u16, div))
    }
}

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
    let (left, middle, right) = split_mut_middle_as_u16(seq);
    reverse_complement_left_right(left, right, tables);

    match middle.len() {
        0 => {}
        1 => middle[0] = tables.cpl8(middle[0]),
        2 => {
            let tmp = tables.cpl8(middle[0]);
            middle[0] = tables.cpl8(middle[1]);
            middle[1] = tmp;
        },
        3 => {
            middle[1] = tables.cpl8(middle[1]);
            let tmp = tables.cpl8(middle[0]);
            middle[0] = tables.cpl8(middle[2]);
            middle[2] = tmp;
        },
        _ => unreachable!()
    }
}

fn file_size(f: &mut File) -> io::Result<usize> {
    Ok(f.metadata()?.len() as usize)
}

fn split_and_reverse<'a>(data: &mut [u8], tables: &Tables) {
    let data = match memchr::memchr(b'\n', data) {
        Some(i) => &mut data[i + 1..],
        None => return,
    };

    match memchr::memchr(b'>', data) {
        Some(i) => {
            let (head, tail) = data.split_at_mut(i);
            rayon::join(|| reverse_complement(head, tables),
                        || split_and_reverse(tail, tables));
        }
        None => reverse_complement(data, tables),
    };
}

fn main() {
    let mut stdin = File::open("/dev/stdin").expect("Could not open /dev/stdin");
    let size = file_size(&mut stdin).unwrap_or(1024 * 1024);
    let mut data = Vec::with_capacity(size + 1);
    stdin.read_to_end(&mut data).unwrap();
    let tables = &Tables::new();

    split_and_reverse(&mut data, tables);
    let stdout = io::stdout();
    stdout.lock().write_all(&data).unwrap();
}
