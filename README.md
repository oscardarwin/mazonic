# TODO:
## Fix backlinks making the level too easy.

## TO RELEASE:
Display level time if available.

### Other
Better color picking for non Icosahedron

## TO RELEASE ANDROID TEST:
Some notes are being skipped when moving fast?

### Daily Challenges
Social Media


## LATER:
Animation on selector screen after game state change


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

# Build for Android:


rustup default stable

rustup target add x86_64-linux-android
rustup target add aarch64-linux-android

cd mazonic_android

cargo ndk -t arm64-v8a -o app/src/main/jniLibs build --package mazonic_android

gradle wrapper


Then:
Launch and run app in Android Studio.
