.PHONY: clean

SOURCES:=$(shell find -type f -iname '*.rs')

all: build

build: $(SOURCES)
	cargo build

release: test $(SOURCES)
	cargo build --release

run: build
	cargo run

test:
	cargo test

clean:
	rm -r ./target
