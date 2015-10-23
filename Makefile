.PHONY: clean

SOURCES:=$(wildcard ./src/*.rs)

all: build

build: $(SOURCES)
	cargo build

release: test $(SOURCES)
	cargo build --release

format: $(SOURCES)
	@for f in $(SOURCES); do rustfmt $$f; done

run: build
	cargo run

test:
	cargo test

clean:
	rm -r ./target
