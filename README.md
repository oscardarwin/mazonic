# TODO:

## Github Pages page

## Graphics
Animation on level change?

## Menu
Play
Daily
Settings
Statistics

## Higher Level Navigation
Menu
- Play
- Settings
- Daily
- Credits

Play 
- 20 levels

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

