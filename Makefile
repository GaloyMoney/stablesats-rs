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
