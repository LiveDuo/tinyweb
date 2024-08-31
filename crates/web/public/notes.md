
### Setup

```sh
cargo build -p example --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/example.wasm crates/web/public/client.wasm
cp js-wasm.js crates/web/public/js-wasm.js
```
