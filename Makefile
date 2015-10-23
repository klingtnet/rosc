.PHONY: clean

SOURCES:=$(wildcard ./src/*.rs)
EXAMPLES:=$(wildcard ./examples/*.rs)

all: build

build: $(SOURCES)
	cargo build

release: test $(SOURCES)
	cargo build --release

format: $(SOURCES) $(EXAMPLES)
	@for f in $(SOURCES) $(EXAMPLES); \
		do echo $$f && rustfmt $$f; \
	done

examples: $(EXAMPLES)

./examples/%.rs:
	cargo build --example $(basename $(notdir $@))

run: build
	cargo run

test:
	cargo test

clean:
	rm -r ./target

doc:
	cargo doc