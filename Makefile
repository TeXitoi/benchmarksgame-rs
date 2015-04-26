SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc
RUSTC_FLAGS ?= -C opt-level=3 -C target-cpu=core2 -C lto -L ./lib
REGEX_VERSION ?= 0.1.30

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

bin/regex_dna: src/regex_dna.rs lib/.regex_install

lib/.regex_install:
	mkdir -p tmp
	curl -s -L https://crates.io/api/v1/crates/regex/$(REGEX_VERSION)/download > tmp/regex-$(REGEX_VERSION).crate
	tar -C tmp/ -xzf tmp/regex-$(REGEX_VERSION).crate
	cargo build --release --manifest-path tmp/regex-$(REGEX_VERSION)/Cargo.toml
	mkdir -p lib
	cp tmp/regex-$(REGEX_VERSION)/target/release/*.rlib lib/
	@touch lib/.regex_install

bin/%: src/%.rs
	mkdir -p bin
	$(RUSTC) $(RUSTC_FLAGS) $< -o $@

out/%.txt: bin/% data/%.txt
	mkdir -p out
	$< < data/$*.txt > $@

diff/%.diff: out/%.txt ref/%.txt
	mkdir -p diff
	diff -u ref/$*.txt $< > $@
