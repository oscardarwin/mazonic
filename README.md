# TODO:


## Save & Load levels as JSON

## Sort out GIT

## More types of levels
Octahedron
Dodecahedron
Icosahedron

## Improve Maze Design
- random walk based design rather than MST.
- more back edges from stranded nodes.
- make the solitary outgoing edge of a node (with a different incoming edge) one way.

## Graphics
Combine connected one way lines into a single dashed line.
Shaders - slighly shiney.

Rounded edges on the shapes?

## Levels selector should be a icosohedron.

## Music

## Effects

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

