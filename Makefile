format:
	@cargo fmt

lint:
	@cargo check

build:
	@cargo build --release

sync:
	@RUST_LOG=INFO ./target/release/ghd sync
