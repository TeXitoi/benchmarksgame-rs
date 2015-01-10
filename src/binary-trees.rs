// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi

extern crate arena;

use std::iter::range_step;
use std::thread::Thread;
use arena::TypedArena;

enum Tree<'a> {
    Nil,
    Node(&'a Tree<'a>, &'a Tree<'a>, int)
}

fn item_check(t: &Tree) -> int {
    match *t {
        Tree::Nil => 0,
        Tree::Node(l, r, i) => i + item_check(l) - item_check(r)
    }
}

fn bottom_up_tree<'r>(arena: &'r TypedArena<Tree<'r>>, item: int, depth: int)
                  -> &'r Tree<'r> {
    if depth > 0 {
        arena.alloc(Tree::Node(bottom_up_tree(arena, 2 * item - 1, depth - 1),
                               bottom_up_tree(arena, 2 * item, depth - 1),
                               item))
    } else {
        arena.alloc(Tree::Nil)
    }
}

fn main() {
    let args = std::os::args();
    let args = args.as_slice();
    let n = std::os::args().get(1).and_then(|n| n.parse()).unwrap_or(10);
    let min_depth = 4;
    let max_depth = if min_depth + 2 > n {min_depth + 2} else {n};

    {
        let arena = TypedArena::new();
        let depth = max_depth + 1;
        let tree = bottom_up_tree(&arena, 0, depth);

        println!("stretch tree of depth {}\t check: {}",
                 depth, item_check(tree));
    }

    let long_lived_arena = TypedArena::new();
    let long_lived_tree = bottom_up_tree(&long_lived_arena, 0, max_depth);

    let messages = range_step(min_depth, max_depth + 1, 2).map(|depth| {
            use std::num::Int;
            let iterations = 2i.pow((max_depth - depth + min_depth) as uint);
            Thread::scoped(move|| {
                let mut chk = 0;
                for i in range(1, iterations + 1) {
                    let arena = TypedArena::new();
                    let a = bottom_up_tree(&arena, i, depth);
                    let b = bottom_up_tree(&arena, -i, depth);
                    chk += item_check(a) + item_check(b);
                }
                format!("{}\t trees of depth {}\t check: {}",
                        iterations * 2, depth, chk)
            })
        }).collect::<Vec<_>>();

    for message in messages.into_iter() {
        println!("{}", message.join().ok().unwrap());
    }

    println!("long lived tree of depth {}\t check: {}",
             max_depth, item_check(long_lived_tree));
}
