bacon:
	bacon check

cargo-test:
	cargo test -- --nocapture

fmt: 
	cargo fmt --all

dev_install: 
	cargo install --path .

ridge:
    skridge
