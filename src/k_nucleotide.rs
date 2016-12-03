// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by Joshua Landau

mod sequence {
    use std::iter;
    use std::slice;

    fn pack_byte(byte: u8, offset: usize) -> u64 {
        // {a: 0, c: 1, t: 2, g: 3}[byte.lower()] << offset
        (((byte >> 1) & 0b11) as u64) << offset
    }

    fn unpack_byte(packed: u64, offset: u8) -> char {
        ['A', 'C', 'T', 'G'][((packed >> offset) & 0b11) as usize]
    }

    pub struct Sequence {
        data: Vec<u64>,
        bit_len: usize,
    }

    impl Sequence {
        pub fn new() -> Sequence {
            Sequence { data: vec![0, 0], bit_len: 0 }
        }

        // Yes, this is hilarious
        pub fn eat(&mut self, cereal: &[u8]) {
            let mut start = self.bit_len;
            let mut working = self.data[start / 64];

            for &byte in cereal {
                // Pack amino acid
                working |= pack_byte(byte, start % 64);
                start += 2;

                if start % 64 == 0 {
                    self.data[start / 64 - 1] = working;
                    working = 0;
                    self.data.push(0);
                    // self.data has two trailing 0s
                }
            }

            self.data[start / 64] = working;
            self.bit_len = start;
        }

        pub fn sub_sequences(&self, size: u8) -> SubSequences {
            if !(0 < size && size <= 32) {
                panic!("Size must be 0 < size <= 32");
            }

            let mut data = self.data.iter().cloned();
            let first = data.next().unwrap();
            SubSequences {
                left: 0,
                right: first,
                data: data,
                shift_out: 64 - (size * 2),
                bit_start: 0,
                bit_end: self.bit_len - (size * 2) as usize,
            }
        }
    }


    #[derive(Copy, Clone, Default, Eq, PartialEq)]
    pub struct SubSequence(u64);

    impl SubSequence {
        pub fn from_bytes(cereal: &[u8]) -> SubSequence {
            let mut sequence = Sequence::new();
            sequence.eat(cereal);
            sequence.sub_sequences(cereal.len() as u8).next().unwrap()
        }

        pub fn hash(&self) -> u64 {
            match *self { SubSequence(hash) => hash }
        }

        pub fn to_string(&self, length: u8) -> String {
            ((32-length)..32).map(|i| unpack_byte(self.hash(), 2 * i)).collect()
        }
    }

    pub struct SubSequences<'a> {
        left: u64,
        right: u64,
        data: iter::Cloned<slice::Iter<'a, u64>>,
        shift_out: u8,
        bit_start: usize,
        bit_end: usize,
    }

    impl<'a> Iterator for SubSequences<'a> {
        type Item = SubSequence;

        fn next(&mut self) -> Option<SubSequence> {
            if !(self.bit_start <= self.bit_end) {
                return None;
            }

            let packed = match self.bit_start % 64 {
                0 => {
                    self.left = self.right;
                    self.right = self.data.next().unwrap();
                    self.left
                },
                shift => (self.left >> shift) | (self.right << (64 - shift))
            };

            self.bit_start += 2;
            Some(SubSequence(packed << self.shift_out))
        }
    }
}


// Can't use the built-in HashMap since custom hashers haven't been stabilized.
mod counter {
    use std::cmp;
    use std::iter;
    use std::slice;
    use sequence::SubSequence;

    const DEFAULT_LOG: u8 = 8;
    const GROWTH_LOG: u8 = 3;
    const BUFFER_LEN: usize = 8;

    #[derive(Copy, Clone, Default)]
    pub struct Entry {
        pub key: SubSequence,
        pub count: usize,
    }

    pub struct Counter {
        slots: Vec<Entry>,
        hash_offset: u8,
    }

    impl Counter {
        pub fn new() -> Counter {
            Counter {
                slots: vec![Default::default(); BUFFER_LEN + (1 << DEFAULT_LOG)],
                hash_offset: 64 - DEFAULT_LOG,
            }
        }

        // Robin Hood Hash
        pub fn increment(&mut self, key: SubSequence) -> usize {
            let mut new = Entry { key: key, count: 1 };

            loop {
                let start = (new.key.hash() >> self.hash_offset) as usize;
                for search_pos in start..(start + 8) {
                    let entry = self.slots[search_pos];

                    if entry.count == 0 {
                        self.slots[search_pos] = new;
                        return 0;
                    }
                    else if entry.key == new.key {
                        self.slots[search_pos].count = entry.count + new.count;
                        return entry.count;
                    }
                    else if entry.key.hash() > new.key.hash() {
                        self.slots[search_pos] = new;
                        new = entry;
                    }
                }

                self.resize();
            }
        }

        #[inline(never)]
        fn resize(&mut self) {
            self.hash_offset -= GROWTH_LOG;
            let len = self.slots.len() - BUFFER_LEN;
            let mut new = vec![Default::default(); (len << GROWTH_LOG) + BUFFER_LEN];

            let mut next_empty = 0;
            for entry in self.iter() {
                let search_pos = entry.key.hash() >> self.hash_offset;
                let insert_at = cmp::max(next_empty, search_pos as usize);
                new[insert_at] = entry;
                next_empty = insert_at + 1;
            }

            self.slots = new;
        }

        pub fn iter(&self) -> Iter {
            Iter(self.slots.iter().cloned())
        }
    }

    pub struct Iter<'a>(iter::Cloned<slice::Iter<'a, Entry>>);

    impl<'a> Iterator for Iter<'a> {
        type Item = Entry;

        fn next(&mut self) -> Option<Entry> {
            self.0.find(|&x| x.count != 0)
        }
    }
}



use counter::Counter;
use sequence::{Sequence, SubSequence};

use std::io;
use std::sync::Arc;
use std::thread;

fn get_sequence<R: std::io::BufRead>(mut input: R, key: &[u8]) -> io::Result<Sequence> {
    let mut sequence = Sequence::new();
    let mut line = Vec::new();

    while !line.starts_with(key) {
        line.truncate(0);
        try!(input.read_until(b'\n', &mut line));
    }

    loop {
        line.truncate(0);
        try!(input.read_until(b'\n', &mut line));
        if line.is_empty() || line.starts_with(b">") {
            break;
        }
        if line.ends_with(b"\n") {
            line.pop();
        }
        sequence.eat(&line);
    }

    Ok(sequence)
}

fn build_counter(sequence: &Arc<Sequence>, size: u8) -> Counter {
    let mut counter = Counter::new();
    for sub_genome in sequence.sub_sequences(size) {
        counter.increment(sub_genome);
    }
    counter
}

fn of_size(sequence: &Arc<Sequence>, size: u8) -> String {
    let counter = build_counter(sequence, size);
    let mut entries: Vec<_> = counter.iter().collect();

    entries.sort_by(|x, y| y.count.cmp(&x.count));
    let total_count = entries.iter().fold(0, |tot, e| tot + e.count);

    entries.iter().map(|entry| format!("{} {:.3}\n",
        entry.key.to_string(size),
        (entry.count as f64) / (total_count as f64) * 100.0
    )).fold(String::new(), |a, b| a + &b)
}

fn of_string(sequence: &Arc<Sequence>, string: &str) -> String {
    let bytes = string.as_bytes();
    let mut counter = build_counter(sequence, bytes.len() as u8);
    let count = counter.increment(SubSequence::from_bytes(bytes));
    format!("{}\t{}", count, string)
}

fn main() {
    let stdin = std::io::stdin();
    let sequence = Arc::new(get_sequence(stdin.lock(), b">THREE").unwrap());
    let mut threads = Vec::new();

    let seq = sequence.clone();
    threads.push(thread::spawn(move || vec![
        of_size(&seq, 1),
        of_size(&seq, 2),
        of_string(&seq, "GGT")
    ]));

    let seq = sequence.clone();
    threads.push(thread::spawn(move || vec![
        of_string(&seq, "GGTA"),
        of_string(&seq, "GGTATT")
    ]));

    let seq = sequence.clone();
    threads.push(thread::spawn(move || vec![
        of_string(&seq, "GGTATTTTAATT")
    ]));

    let seq = sequence.clone();
    threads.push(thread::spawn(move || vec![
        of_string(&seq, "GGTATTTTAATTTATAGT")
    ]));

    for thread in threads {
        for line in thread.join().unwrap() {
            println!("{}", line);
        }
    }
}
