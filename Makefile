.PHONY: clean

SOURCES:=$(wildcard ./src/*.rs)
TESTS:=$(wildcard ./tests/*.rs)
EXAMPLES:=$(wildcard ./examples/*.rs)
BUILD_OPTS:=--jobs $(shell nproc)

all: build

build: $(SOURCES)
	cargo build $(BUILD_OPTS)

release: test $(SOURCES)
	cargo build --release $(BUILD_OPTS)

format: $(SOURCES) $(EXAMPLES) $(TESTS)
	@for f in $?; \
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
	rm -f src/*.rs.bk

doc: $(SOURCES)
	cargo doc
