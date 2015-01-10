SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc

.PHONY: all distclean clean

all: $(patsubst src/%.rs,diff/%.diff, $(SOURCES))

clean:
	rm -fr diff

distclean: clean
	rm -fr bin out

bin/%: src/%.rs
	mkdir -p bin
	$(RUSTC) -C opt-level=3 -C target-cpu=core2 -C lto $< -o $@

out/%.txt: bin/% data/%.txt
	mkdir -p out
	$< < data/$*.txt > $@

diff/%.diff: out/%.txt ref/%.txt
	mkdir -p diff
	diff -u ref/$*.txt $< > $@
