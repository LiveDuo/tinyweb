build:
	cargo build --target wasm32-unknown-unknown -r
	mkdir -p /tmp/public
	cp ../../target/wasm32-unknown-unknown/release/features.wasm /tmp/public/features.wasm
	cp ../../src/js/main.js /tmp/public/main.js
	cp public/index.html /tmp/public/index.html
start:
	python3 -m http.server -d /tmp/public
dev:
	make build
	make start
