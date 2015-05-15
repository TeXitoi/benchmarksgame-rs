// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

fn start(n_tasks: i32, token: i32) {
    let (tx, mut rx) = channel();
    tx.send(token).unwrap();
    let mut guards = Vec::with_capacity(n_tasks as usize);
    for i in 2 .. n_tasks + 1 {
        let (tx, next_rx) = channel();
        let cur_rx = std::mem::replace(&mut rx, next_rx);
        guards.push(thread::spawn(move|| roundtrip(i, tx, cur_rx)));
    }
    guards.push(thread::spawn(move|| roundtrip(1, tx, rx)));
    for g in guards { g.join().unwrap(); }
}

fn roundtrip(id: i32, tx: Sender<i32>, rx: Receiver<i32>) {
    for token in rx.iter() {
        if token == 1 {
            println!("{}", id);
            break;
        }
        tx.send(token - 1).unwrap();
    }
}

fn main() {
    let args = &mut std::env::args_os();
    let token = args.skip(1).next()
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(1000);
    let n_tasks = args.next()
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(503);
    start(n_tasks, token);
}
