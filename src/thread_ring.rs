// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by Joshua Landau

// Custom locks for 2-stage locking
mod locks {
    use std::sync::{Condvar, Mutex};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    pub struct Lock {
        condvar: Condvar,
        is_set: Mutex<bool>
    }

    impl Lock {
        pub fn new(unlocked: bool) -> Lock {
            Lock { condvar: Condvar::new(), is_set: Mutex::new(unlocked) }
        }

        pub fn lock(&self) {
            let mut set = self.is_set.lock().unwrap();
            while !*set {
                set = self.condvar.wait(set).unwrap();
            }
            *set = false;
        }

        pub fn unlock(&self) {
            let mut set = self.is_set.lock().unwrap();
            *set = true;
            self.condvar.notify_one();
        }
    }

    const EMPTY: usize = ::std::usize::MAX;
    pub struct SpinLock(AtomicUsize);

    impl SpinLock {
        pub fn new(value: Option<usize>) -> SpinLock {
            SpinLock(AtomicUsize::new(value.unwrap_or(EMPTY)))
        }

        pub fn lock(&self) -> usize {
            loop {
                let gotten = self.0.swap(EMPTY, Ordering::SeqCst);
                if gotten != EMPTY {
                    return gotten;
                }
                thread::yield_now();
            }
        }

        pub fn unlock(&self, value: usize) {
            self.0.store(value, Ordering::SeqCst);
        }
    }
}

use std::sync::Arc;
use std::thread;

use locks::{Lock, SpinLock};

fn start(n_tasks: usize, token: usize) {
    let locks: Vec<_> = (0..n_tasks).map(|i|
        Arc::new(Lock::new(i == 1 || i == 2))
    ).collect();

    let io: Vec<_> = (0..n_tasks).map(|i|
        Arc::new(SpinLock::new(if i == 1 { Some(token) } else { None }))
    ).collect();

    let threads: Vec<_> = (0..n_tasks).map(|i| {
        let lock   = locks[i].clone();
        let input  = io[i].clone();
        let output = io[(i + 1) % n_tasks].clone();
        let unlock = locks[(i + 2) % n_tasks].clone();

        thread::spawn(move || roundtrip(i + 1, lock, input, output, unlock))
    }).collect();

    for thread in threads {
        thread.join().unwrap();
    }
}

fn roundtrip(
    thread_id: usize,
    lock:   Arc<Lock>,
    input:  Arc<SpinLock>,
    output: Arc<SpinLock>,
    unlock: Arc<Lock>,
) {
    loop {
        lock.lock();
        let input_value = input.lock();
        output.unlock(input_value.saturating_sub(1));
        unlock.unlock();

        if input_value == 1 { println!("{}", thread_id); }
        if input_value <= 1 { return; }
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
