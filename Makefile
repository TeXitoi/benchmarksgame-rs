SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc
RUSTC_FLAGS ?= -C opt-level=3 -C target-cpu=core2 -C lto
RUSTC_FLAGS += -L ./lib
REGEX ?= regex-0.1.30
ARENA ?= typed-arena-1.0.1

.PHONY: all distclean clean
.SECONDARY:

all: $(patsubst src/%.rs,diff/%.diff, $(SOURCES))

clean:
	rm -fr diff

distclean: clean
	rm -fr bin out tmp lib

diff/chameneos_redux.diff: out/chameneos_redux.txt ref/chameneos_redux.txt
	mkdir -p diff
	sed -r 's/^[0-9]+/42/' $< | diff -u ref/chameneos_redux.txt - > $@

bin/regex_dna: src/regex_dna.rs lib/$(REGEX).pkg

bin/binary_trees: src/binary_trees.rs lib/$(ARENA).pkg

lib/%.pkg:
	mkdir -p tmp
	curl -s -L https://crates.io/api/v1/crates/`echo '$*' | sed -r 's#-([^-]*$$)#/\1#'`/download > tmp/$*.crate
	tar -C tmp/ -xzf tmp/$*.crate
	cargo build --release --manifest-path tmp/$*/Cargo.toml
	mkdir -p lib
	cp tmp/$*/target/release/*.rlib lib/
	@touch lib/$*.pkg

bin/%: src/%.rs
	mkdir -p bin
	$(RUSTC) $(RUSTC_FLAGS) $< -o $@

out/%.txt: bin/% data/%.txt
	mkdir -p out
	$< < data/$*.txt > $@

diff/%.diff: out/%.txt ref/%.txt
	mkdir -p diff
	diff -u ref/$*.txt $< > $@
