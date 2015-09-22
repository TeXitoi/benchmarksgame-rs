SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc
RUSTC_FLAGS ?= -C opt-level=3 -C target-cpu=core2 -C lto
RUSTC_FLAGS += -L ./lib
REGEX ?= regex
ARENA ?= typed-arena

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

bin/binary_trees: src/binary_trees.rs lib/$(ARENA).pkg

bin/regex_dna: src/regex_dna.rs lib/regex.pkg

lib/%.pkg:
	mkdir -p tmp
	rm -rf tmp/$*-deps
	cargo new tmp/$*-deps
	@echo '\n[dependencies]\n$* = "*"' >> tmp/$*-deps/Cargo.toml
	cargo build --release --manifest-path tmp/$*-deps/Cargo.toml
	mkdir -p lib
	cp tmp/$*-deps/target/release/deps/* lib/
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
