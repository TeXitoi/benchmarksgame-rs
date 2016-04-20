SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc
RUSTC_FLAGS ?= -C opt-level=3 -C target-cpu=core2 -C lto
RUSTC_FLAGS += -L ./lib
REGEX ?= regex-0.1.41
ARENA ?= typed-arena-1.0.1
NUM_CPU ?= num_cpus-0.2.6

version=$(lastword $(subst -,  , $1))
crate=$(strip $(subst -$(call version, $1),, $1))

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

bin/fasta: src/fasta.rs lib/$(NUM_CPU).pkg

bin/fasta_redux: src/fasta_redux.rs lib/$(NUM_CPU).pkg

bin/regex_dna: src/regex_dna.rs lib/$(REGEX).pkg

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
