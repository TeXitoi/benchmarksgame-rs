// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by Matt Brubeck
// contributed by TeXitoi

extern crate typed_arena;
extern crate rayon;

use typed_arena::Arena;
use rayon::prelude::*;

struct Tree<'a> {
    l: Option<&'a Tree<'a>>,
    r: Option<&'a Tree<'a>>,
    i: i32
}

fn item_check(t: Option<&Tree>) -> i32 {
    match t {
        None => 0,
        Some(&Tree { l, r, i }) => i + item_check(l) - item_check(r)
    }
}

fn bottom_up_tree<'r>(arena: &'r Arena<Tree<'r>>, item: i32, depth: i32)
                  -> Option<&'r Tree<'r>> {
    if depth > 0 {
        let t: &Tree<'r> = arena.alloc(Tree {
            l: bottom_up_tree(arena, 2 * item - 1, depth - 1),
            r: bottom_up_tree(arena, 2 * item, depth - 1),
            i: item
        });
        Some(t)
    } else {
        None
    }
}

fn inner(depth: i32, iterations: i32) -> String {
    let chk = (1 .. iterations + 1).into_par_iter().map(|i| {
        let arena = Arena::new();
        let a = bottom_up_tree(&arena, i, depth);
        let b = bottom_up_tree(&arena, -i, depth);
        item_check(a) + item_check(b)
    }).sum();
    format!("{}\t trees of depth {}\t check: {}",
            iterations * 2, depth, chk)
}

fn main() {
    let n = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(10);
    let min_depth = 4;
    let max_depth = if min_depth + 2 > n {min_depth + 2} else {n};

    {
        let arena = Arena::new();
        let depth = max_depth + 1;
        let tree = bottom_up_tree(&arena, 0, depth);

        println!("stretch tree of depth {}\t check: {}",
                 depth, item_check(tree));
    }

    let long_lived_arena = Arena::new();
    let long_lived_tree = bottom_up_tree(&long_lived_arena, 0, max_depth);

    let messages = (min_depth/2..max_depth/2 + 1)
            .into_par_iter()
            .weight_max()
            .map(|half_depth| {
                let depth = half_depth * 2;
                let iterations = 1 << ((max_depth - depth + min_depth) as u32);
                inner(depth, iterations)
            }).collect::<Vec<_>>();

    for message in messages.into_iter() {
        println!("{}", message);
    }

    println!("long lived tree of depth {}\t check: {}",
             max_depth, item_check(long_lived_tree));
}
