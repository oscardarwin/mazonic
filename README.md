# TODO:

## Improve Maze Design
- both of these in "loops".

- Random sample edges with a loose node as "from" and not existing in graph yet.
- From these the nodes must be "far apart in distance" and "big differences between their target node distance."

- Random sample edges that are entering intersections (Neighbors of from node >= 2).
- Check whether maze is still solvable from the "to node" using Astar with a massive weight on the edge. 
- If so, remove the edge.

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

