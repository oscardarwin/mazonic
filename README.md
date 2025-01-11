# TODO:

## Github Pages page

## Graphics
Camera adjust to screen size.
loading animation.

Combine connected one way lines into a single dashed line.
Shaders - slighly shiney.

Rounded edges on the shapes?
Animation on level change?

Different colours on each side as a difficulty heuristic.

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

