.PHONY: build dist setup-cross check clean

build:
	cargo build --release

# One-time setup. Docker or Podman must also be installed and running.
setup-cross:
	cargo install cross --git https://github.com/cross-rs/cross --locked

dist:
	./scripts/build-all.sh

check:
	cargo test
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
	rm -rf dist
