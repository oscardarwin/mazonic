# TODO:

## Improve Maze Design
- more back edges from stranded nodes.
- make the solitary outgoing edge of a node (with a different incoming edge) one way.

## More types of levels
Tetrahedron
Octahedron
Dodecahedron
Icosahedron

## Save levels as JSON

## Sort out GIT

## Graphics
Add death node for any stranded edges.
Combine connected one way lines into a single dashed line.
Shaders

## Levels selector should be a icosohedron.

## Music

## Effects

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

