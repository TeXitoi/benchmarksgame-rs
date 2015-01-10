# The Computer Language Benchmarks Game: Rust implementations [![Build Status](https://travis-ci.org/TeXitoi/benchmarksgame-rs.svg?branch=master)](https://travis-ci.org/TeXitoi/benchmarksgame-rs)

This is the version I propose to the [The Computer Language Benchmarks
Game](http://benchmarksgame.alioth.debian.org/).  It is mainly [the
official rust
versions](https://github.com/rust-lang/rust/tree/master/src/test/bench)
to which I made the following modifications to be published:
 - changing the header, as asked by the site (I have a dedicated
   contributor line because I propose the program as asked by the
   site);
 - removing the rust test framework specific code;
 - possibly removing warnings or other *trivial* modifications.

If you want to contribute, please propose your version first to the
official rust repository, and then propose a pull request here.

There is a specific exception: pidigits.  I have a special version
that is not provided by the rust repo because it depends on GMP.
