# TODO:

## Github Pages page

## Graphics
Glow on player
Animation on level change?

Make global shader with discrete

Shaders - slighly shiney.

## Menu
Play
Daily
Settings
Statistics

## Levels selector should be a icosohedron.

## Music
Ping sound effect

## Effects


# Name Drafts
Hedron

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

