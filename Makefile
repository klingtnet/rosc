.PHONY: clean

SOURCES:=$(wildcard ./src/*.rs)
TESTS:=$(wildcard ./tests/*.rs)
EXAMPLES:=$(wildcard ./examples/*.rs)
BUILD_OPTS:=--jobs $(shell nproc)

all: test build examples doc

build: $(SOURCES)
	cargo build $(BUILD_OPTS)

release: test $(SOURCES)
	cargo build --release $(BUILD_OPTS)

fmt: format

format: $(SOURCES) $(EXAMPLES) $(TESTS)
	@for f in $?; do\
		echo $$f && rustfmt $$f; \
	done

examples: $(SOURCES) $(EXAMPLES)
	@for f in $(basename $(notdir $(EXAMPLES))); do\
		cargo build --example $$f; \
	done

run: build
	cargo run

test: $(TESTS) $(SOURCES)
	cargo test

clean:
	rm -r ./target
	rm -f src/*.rs.bk

doc: $(SOURCES)
	cargo doc
