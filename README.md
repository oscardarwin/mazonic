# TODO:


Two modes while clicked:

Node on point:
Project all neighbour nodes onto the same face. 
Calculate Direction Vectors to each neighbour node.
Pick the node with the minimum angle to each neighbour node.

Node now on path:

# Cross Face Edge Idea

project point from mouse onto plane at player level on both faces. Decide which plane based on which projected point is closer to origin.



# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

