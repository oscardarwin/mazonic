# TODO:

## Improve Maze Design
- Add one way edges.
- Randomly choose when to use one way edges.
- Prioritise long branches.
- One way edges back to earlier edges in the solution.
- Delete some nodes to not have too many long edges.

## Graphics
Treasure map style.
No node when there is just an in and out of edge.

## More types of levels
## Levels selector should be a icosohedron.

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

