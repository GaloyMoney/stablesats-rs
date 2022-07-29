watch:
	RUST_BACKTRACE=full cargo watch -s 'cargo test --all-features -- --nocapture'

next-watch:
	RUST_BACKTRACE=full cargo watch -s 'cargo nextest run --all-features'

test-in-ci:
	cargo clippy --all-features
	cargo nextest run --all-features --verbose --locked
