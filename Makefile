watch:
	RUST_BACKTRACE=full cargo watch -s 'cargo test --all-features -- --nocapture'

next-watch:
	RUST_BACKTRACE=full cargo watch -s 'cargo nextest run --all-features'
