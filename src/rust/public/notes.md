
### Setup

```sh
cargo build -p example --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/example.wasm src/rust/public/client.wasm
cp src/js/js-wasm.js src/rust/public/js-wasm.js
```
