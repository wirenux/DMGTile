.PHONY: all mac mac-intel windows linux clean run

all: mac

mac:
	cargo build --release

mac-intel:
	cargo build --release --target x86_64-apple-darwin

windows:
	RUSTFLAGS="-C target-feature=+crt-static" \
		cargo build --release --target x86_64-pc-windows-gnu

linux:
	cross build --release --target x86_64-unknown-linux-gnu

run:
	cargo run

clean:
	cargo clean
