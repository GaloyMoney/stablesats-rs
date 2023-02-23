build:
	cargo build

watch:
	RUST_BACKTRACE=full cargo watch -s 'cargo test -- --nocapture'

next-watch:
	cargo watch -s 'cargo nextest run'

check-code:
	SQLX_OFFLINE=true cargo fmt --check --all
	SQLX_OFFLINE=true cargo clippy --all-features
	SQLX_OFFLINE=true cargo audit

test-in-ci:
	DATABASE_URL=postgres://user:password@postgres:5432/pg cargo sqlx migrate run
	SQLX_OFFLINE=true cargo nextest run --verbose --locked

cli-run:
	SQLX_OFFLINE=true cargo run --bin stablesats run

build-x86_64-unknown-linux-musl-release:
	SQLX_OFFLINE=true cargo build --release --locked --target x86_64-unknown-linux-musl

build-x86_64-apple-darwin-release:
	bin/osxcross-compile.sh

clean-deps:
	docker compose down

start-deps:
	docker compose up -d integration-deps

start-deps-local:
	docker compose up -d postgres

reset-deps: clean-deps start-deps setup-db

reset-deps-local: clean-deps start-deps-local setup-db

setup-db:
	cargo sqlx migrate run
