// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by Joshua Landau

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;


const DIGITS: [&'static str; 10] = [
    "zero", "one", "two", "three", "four",
    "five", "six", "seven", "eight", "nine",
];

fn wordy_num(num: usize) -> String {
    let mut out = String::new();
    for char in num.to_string().chars() {
        out.push_str(" ");
        out.push_str(DIGITS[char.to_digit(10).unwrap() as usize]);
    }
    out
}


#[derive(Clone, Copy)]
#[repr(u8)]
enum Color {
    Red = 0,
    Yellow = 1,
    Blue = 2,
}

impl Color {
    fn show(&self) -> &'static str {
        use Color::*;
        match *self {
            Red => "red",
            Yellow => "yellow",
            Blue => "blue",
        }
    }
}

fn complement_color(left: Color, right: Color) -> Color {
    use Color::*;
    match ((left as u8) << 2) | (right as u8) {
        0b_00_00 => Red,
        0b_00_01 => Blue,
        0b_00_10 => Yellow,
        0b_01_00 => Blue,
        0b_01_01 => Yellow,
        0b_01_10 => Red,
        0b_10_00 => Yellow,
        0b_10_01 => Red,
        _        => Blue,
    }
}


#[derive(Default)]
struct AtomicColor(AtomicUsize);

impl AtomicColor {
    fn load(&self, order: Ordering) -> Color {
        use Color::*;
        match self.0.load(order) {
            0 => Red,
            1 => Yellow,
            _ => Blue,
        }
    }

    fn store(&self, color: Color, order: Ordering) {
        self.0.store(color as usize, order)
    }
}


// Each Chameneos is atomic to allow safe, fast
// parallel thread access. Unfortunately this
// this is a bit wordy, but it works out OK.
#[derive(Default)]
struct ChameneosState {
    name: u8,
    color: AtomicColor,
    meet_count: AtomicUsize,
    meet_same_count: AtomicUsize,
}

impl ChameneosState {
    fn name(&self) -> u8 {
        self.name
    }

    fn color(&self) -> Color {
        self.color.load(Ordering::Acquire)
    }

    fn meet(&self, same: bool, color: Color) {
        let new = self.meet_count.load(Ordering::Acquire) + 1;
        self.meet_count.store(new, Ordering::Release);
        if same {
            let new = self.meet_same_count.load(Ordering::Acquire) + 1;
            self.meet_same_count.store(new, Ordering::Release);
        }
        self.color.store(color, Ordering::Release);
    }
}


#[derive(Copy, Clone)]
struct Chameneos {
    idx: u32
}

impl Chameneos {
    fn is_valid(&self) -> bool {
        self.idx != 0
    }

    fn get<'st>(&self, shared: &'st Shared) -> &'st ChameneosState {
        &shared.states[(self.idx & BLOCK) as usize]
    }
}


struct Shared {
    // We can only store 12 due to overflow
    // when the maximum number of pairs are
    // created, so no need for a large buffer.
    // Using 16 avoids bounds checks.
    states: [ChameneosState; 16],
    // Bottom block is mall, rest are queue slots
    atomic_queue: AtomicUsize,
    meetings_had: AtomicUsize,
    meetings_limit: usize,
}

impl Shared {
    fn null_task(&self) -> Chameneos {
        self.task_at(0)
    }

    fn task_at(&self, idx: u32) -> Chameneos {
        Chameneos { idx: idx }
    }

    fn load(&self, order: Ordering) -> u32 {
        self.atomic_queue.load(order) as u32
    }

    fn store(&self, val: u32, order: Ordering) {
        self.atomic_queue.store(val as usize, order)
    }

    fn compare_and_swap(&self, current: u32, new: u32, order: Ordering) -> u32 {
        self.atomic_queue.compare_and_swap(current as usize, new as usize, order) as u32
    }
}


const BLOCK: u32 = 0b1111;
const BLOCK_LEN: u32 = 4;
const QUEUE_LEN: u32 = 32 / BLOCK_LEN;
const QUEUE_STOPPED: u32 = !0;

struct State {
    cache: u32,
}

impl State {
    fn new(shared: &Shared) -> State {
        State { cache: shared.load(Ordering::SeqCst) }
    }

    fn run(&mut self, shared: &Shared) -> Option<(TransactionalQueue, Chameneos, u8)> {
        let cache = &mut self.cache;

        if *cache == QUEUE_STOPPED {
            None
        }
        else {
            let mut queue = TransactionalQueue {
                set_state: *cache,
                cache: cache
            };
            let count = queue.take_count();
            let mall = queue.take(shared);
            Some((queue, mall, count))
        }
    }

    fn register_meeting(&mut self, shared: &Shared, count: u8) -> bool {
        if count != 0 { return true; }

        let meetings_had = shared.meetings_had.fetch_add(1, Ordering::Acquire);
        if meetings_had < shared.meetings_limit {
            return true;
        }
        // Oops, we couldn't actually do that
        shared.store(QUEUE_STOPPED, Ordering::SeqCst);
        false
    }
}


struct TransactionalQueue<'a> {
    set_state: u32,
    cache: &'a mut u32,
}

impl<'a> TransactionalQueue<'a> {
    fn submit(mut self, shared: &Shared, mall: Chameneos, count: u8) -> bool {
        self.set_state <<= BLOCK_LEN;
        self.set_state |= mall.idx;

        self.set_state <<= 8;
        self.set_state |= count as u32;

        let actual = shared.compare_and_swap(
            *self.cache,     // expected current value
            self.set_state,  // wanted value
            Ordering::Release
        );

        let worked = actual == *self.cache;
        *self.cache = if worked { self.set_state } else { actual };
        worked
    }

    fn cancel(&mut self, shared: &Shared) {
        *self.cache = shared.load(Ordering::Relaxed);
    }

    fn take_count(&mut self) -> u8 {
        let ret = (self.set_state & 0b11111111) as u8;
        self.set_state >>= 8;
        ret
    }

    fn take(&mut self, shared: &Shared) -> Chameneos {
        let ret = self.set_state & BLOCK;
        self.set_state >>= BLOCK_LEN;
        shared.task_at(ret)
    }

    fn put(&mut self, first: Chameneos, second: Chameneos) {
        let zeros = self.set_state.leading_zeros();
        let shift = (QUEUE_LEN - (zeros / BLOCK_LEN)) * BLOCK_LEN;
        self.set_state |= ((first.idx << BLOCK_LEN) | second.idx) << shift;
    }
}


// Runs threads from the shared thread pool.
// Uses optimistic concurrency to queue and
// deque threads, as well as to take from the
// mall.
fn thread_executor(mut task: Chameneos, shared: &Shared) {
    let mut state = State::new(shared);
    let mut mall = shared.null_task();

    loop {
        let mut task_tmp = task;
        let mut mall_tmp = mall;
        let mut count;

        {
            let (mut queue, new_mall, new_count) = match state.run(shared) {
                Some(x) => x,
                None => return,
            };
            count = new_count;

            if mall_tmp.is_valid() {
                queue.put(task_tmp, mall_tmp);
                task_tmp = queue.take(shared);
            }
            else if !task_tmp.is_valid() {
                task_tmp = queue.take(shared);
                if !task_tmp.is_valid() {
                    std::thread::sleep_ms(1);
                    queue.cancel(shared);
                    continue;
                }
            }

            mall_tmp = new_mall;
            if !mall_tmp.is_valid() {
                let replacement_task = queue.take(shared);
                if queue.submit(shared, task_tmp, count) {
                    task = replacement_task;
                    mall = shared.null_task();
                }
                continue;
            }

            count = count.wrapping_sub(1);
            if !queue.submit(shared, shared.null_task(), count) {
                continue;
            }
            task = task_tmp;
            mall = mall_tmp;
        }

        let actor_ref = task_tmp.get(shared);
        let mall_ref = mall.get(shared);

        let same = actor_ref.name() == mall_ref.name();
        let new_color = complement_color(actor_ref.color(), mall_ref.color());

        actor_ref.meet(same, new_color);
        mall_ref.meet(same, new_color);

        if !state.register_meeting(shared, count) { return; }
    }
}


fn run_for(meetings_limit: usize, colors: &[Color]) -> Vec<(usize, usize)> {
    assert!(meetings_limit != 0);
    let num_threads = colors.len();

    let x = || Default::default();
    let mut states: [ChameneosState; 16] = [
        x(), x(), x(), x(), x(), x(), x(), x(),
        x(), x(), x(), x(), x(), x(), x(), x(),
    ];

    for (i, &color) in colors.iter().enumerate() {
        let idx = i + 1;
        let chameneos_state = &mut states[idx];
        chameneos_state.name = idx as u8;
        chameneos_state.color.store(color, Ordering::Release);
    }

    let shared = Arc::new(Shared {
        atomic_queue: AtomicUsize::new(meetings_limit as u8 as usize),
        meetings_had: AtomicUsize::new(0),
        meetings_limit: ((meetings_limit - 1) >> 8),
        states: states,
    });

    let threads: Vec<_> = (0..num_threads).map(|i| {
        let task = shared.task_at((i + 1) as u32);
        let shared = shared.clone();
        thread::spawn(move || thread_executor(task, &shared))
    }).collect();

    for thread in threads {
        thread.join().unwrap();
    }

    let output = &shared.states[..];

    output[1..colors.len() + 1].iter().map(|ch| (
        ch.meet_count.load(Ordering::SeqCst),
        ch.meet_same_count.load(Ordering::SeqCst),
    )).collect()
}


fn main() {
    use Color::*;
    let small = [Blue, Red, Yellow];
    let large = [Blue, Red, Yellow, Red, Yellow, Blue, Red, Yellow, Red, Blue];

    let num_meetings = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(600);

    let colors = [Blue, Red, Yellow];
    for &left in &colors {
        for &right in &colors {
            let complement = complement_color(left, right);
            println!("{} + {} -> {}", left.show(), right.show(), complement.show());
        }
    }

    let threads: Vec<(&[_], _)> = vec![
        (&small, thread::spawn(move || run_for(num_meetings, &small))),
        (&large, thread::spawn(move || run_for(num_meetings, &large))),
    ];

    for (colors, thread) in threads {
        println!("");

        for color in colors { print!(" {}", color.show()); }
        println!("");

        let mut total_count = 0;
        for (meet_count, meet_same_count) in thread.join().unwrap() {
            println!("{}{}", meet_count, wordy_num(meet_same_count));
            total_count += meet_count;
        }

        println!("{}", wordy_num(total_count));
    }

    println!("");
}
