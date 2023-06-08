bacon:
	bacon check

cargo-test:
	cargo test -- --nocapture

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
