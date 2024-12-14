# TODO:

## Improve Game Feel
camera that rotates around a little as you move.
smooth player movement.

## Levels selector should be a icosohedron.

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

