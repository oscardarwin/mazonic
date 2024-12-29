# TODO:

## Controls
- add arrow key indicators
- make camera lock positions depend on the shape.
- give the player a bit of velocity when you let go.

## Graphicsal
Combine connected one way lines into a single dashed line.
Shaders - slighly shiney.

Rounded edges on the shapes?
Animation on level change?

## Menu

Play
Daily
Settings
Statistics


## Levels selector should be a icosohedron.

## Music

## Effects

## Save & Load levels as JSON

# Name Drafts

Hedron

Polymaze
Mazonic
Puzzlehedron


# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

