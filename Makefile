SOURCES = $(wildcard src/*.rs)
RUSTC ?= rustc
RUSTC_FLAGS ?= -C opt-level=3 -C target-cpu=core2 -C lto

.PHONY: all distclean clean
.SECONDARY:

all: $(patsubst src/%.rs,diff/%.diff, $(SOURCES))

clean:
	rm -fr diff

distclean: clean
	rm -fr bin out

diff/chameneos_redux.diff: out/chameneos_redux.txt ref/chameneos_redux.txt
	mkdir -p diff
	sed -r 's/^[0-9]+/42/' $< | diff -u ref/chameneos_redux.txt - > $@

bin/%: src/%.rs
	mkdir -p bin
	$(RUSTC) $(RUSTC_FLAGS) $< -o $@

out/%.txt: bin/% data/%.txt
	mkdir -p out
	$< < data/$*.txt > $@

diff/%.diff: out/%.txt ref/%.txt
	mkdir -p diff
	diff -u ref/$*.txt $< > $@
