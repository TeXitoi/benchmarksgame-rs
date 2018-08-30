SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc
RUSTC_FLAGS ?= -C opt-level=3 -C target-cpu=core2 -C lto
RUSTC_FLAGS += -L ./lib
REGEX ?= regex-1.0.4
ARENA ?= typed-arena-1.4.1
FUTURES_CPUPOOL ?= futures-cpupool-0.1.8
RAYON ?= rayon-1.0.2
INDEXMAP ?= indexmap-1.0.1
CROSSBEAM ?= crossbeam-0.4.1

version=$(lastword $(subst -,  , $1))
crate=$(strip $(subst -$(call version, $1),, $1))

.PHONY: all distclean clean
.SECONDARY:

all: $(patsubst src/%.rs,diff/%.diff, $(SOURCES))

clean:
	rm -fr diff
distclean: clean
	rm -fr bin out tmp lib

bin/binary_trees: lib/$(ARENA).pkg lib/$(RAYON).pkg
bin/fannkuch_redux: lib/$(RAYON).pkg
bin/k_nucleotide: lib/$(FUTURES_CPUPOOL).pkg lib/$(INDEXMAP).pkg
bin/mandelbrot: lib/$(RAYON).pkg
bin/regex_redux: lib/$(REGEX).pkg
bin/reverse_complement: lib/$(RAYON).pkg
bin/spectralnorm: lib/$(RAYON).pkg

diff/chameneos_redux.diff: out/chameneos_redux.txt ref/chameneos_redux.txt
	mkdir -p diff
	sed -r 's/^[0-9]+/42/' $< | diff -u ref/chameneos_redux.txt - > $@

lib/%.pkg:
	mkdir -p tmp
	rm -rf tmp/$(call crate,$*)-deps
	cargo new tmp/$(call crate,$*)-deps
	printf '$(call crate,$*) = "$(call version,$*)"\n' >> tmp/$(call crate,$*)-deps/Cargo.toml
	cargo build --release --manifest-path tmp/$(call crate,$*)-deps/Cargo.toml
	mkdir -p lib
	cp tmp/$(call crate,$*)-deps/target/release/deps/* lib/
	touch lib/$*.pkg

bin/%: src/%.rs
	mkdir -p bin
	$(RUSTC) $(RUSTC_FLAGS) $< -o $@

out/%.txt: bin/% data/%.txt
	mkdir -p out
	$< < data/$*.txt > $@

diff/%.diff: out/%.txt ref/%.txt
	mkdir -p diff
	diff -u ref/$*.txt $< > $@
