bacon:
	bacon check


fmt: 
	cargo fmt --all

dev_install: 
	cargo install --path .

cargo-watch-test:
	cargo watch -x 'test -- --nocapture'

shell:
	zsh

ridge:
    skridge

cargo-check:
	cargo check --all

cargo-test:
	cargo test --all -- --nocapture

cargo-fmt-check:
	cargo fmt --all --check

cargo-clippy-check:
	cargo clippy --all

pre-push-check: cargo-check cargo-fmt-check cargo-clippy-check cargo-test
