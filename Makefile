build:
	cargo build

watch:
	RUST_BACKTRACE=full cargo watch -s 'cargo test -- --nocapture'

next-watch:
	cargo watch -s 'cargo nextest run'

check-code:
	cargo fmt --check --all
	cargo clippy --all-features
	cargo audit

test-in-ci:
	cargo nextest run --all-features --verbose --locked

cli-run:
	cargo run --bin stablesats run

build-x86_64-unknown-linux-musl-release:
	cargo build --release --locked --target x86_64-unknown-linux-musl

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh
