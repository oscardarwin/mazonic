# TODO:
re-assign materials on Melody found
add gold note particles to completed melody level selector triangles
add gold lettering to melody found triangles

Main Menu
Test out android build?

add remaining hidden melodies.

Social Media?

Fix backlinks making the level too easy.
Make some nodes inaccessable

UI popup on level complete? / Melody found

smoother symbols in UI.

better sounding marimba
reverb?

## Menu
Play
Daily
Time Challenge
Settings - Dodecahedron?
Statistics - Icosahedron
Credits

## Selector
Perfect Score - Blue - Particles
Hidden Melody - Gold Symbol - Particles (with quaver symbols)

## Possible Polyhedra vertex numbers:

formula:
N * (N + 1) = 2, 6, 12, 20, 30, 42, 56, 72, 90, 110, 132

2 * N * (N + 1) = 4, 12, 24, 40, 60, 84, 112, 144, 180, 220, 264
6 * N * N = 6, 24, 54, 96, 150, 216, 294, 384, 
4 * N * (N + 1) = 8, 24, 48, 80, 120, 168, 224, 288, 360
Dodecahedron: 60
10 * N * (N + 1) = 20, 60, 120, 200, 300, 420, 560, 720

Tetrahedron 1 = 4
Cube 2 = 24
Octahedron 3 = 48
Dodecahedron 60
Icosahedron 2 = 60
Octahedron 4 = 80
Tetrahedron 6 = 84
Cube 4 = 96
Tetrahedron 7 = 112
Octahedron 5 = 120
Icosahedron 3 = 120
Tetrahedron 8 = 144
Cube 5 = 150
Octahedron 6 = 168
Tetrahedron 9 = 180
Icosahedron 4 = 200
Cube 6 = 216
Octahedron 7 = 224
Cube 7 = 296
Icosahedron 5 = 300

# Build for Web:

install wasm32-unknown-unknown

## Commands
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/mazonic.wasm
npx serve web

