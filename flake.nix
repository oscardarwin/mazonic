{
  description = "Generate mazes";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    android-nixpkgs.url = "github:tadfisher/android-nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs@{ flake-parts, android-nixpkgs, nixpkgs, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { lib, system, ... }:
        let
          overlays = [ rust-overlay.overlays.default ];
          pkgs = import nixpkgs { inherit system overlays; };

          android-sdk = android-nixpkgs.sdk.${system} (sdkPkgs: with sdkPkgs; [
            cmdline-tools-latest
            build-tools-34-0-0
            platform-tools
            platforms-android-34
            emulator
            ndk-26-1-10909125
          ]);

          rust = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-std" "cargo" ];
            targets = [ "aarch64-linux-android" ];
          };

        in
        {
          devShells.default = pkgs.mkShell rec {
            buildInputs = with pkgs; [
              rustfmt
              pkg-config
              gdb
              openssl
              libGL

              wayland
              udev
              vulkan-loader
              alsa-lib
              vulkan-loader
              libxkbcommon
              stdenv.cc.cc.lib

              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr

              rust
              rustup
              kotlin
              gradle
              nodejs_22
              xsel

              aapt # still needed?
              jdk # remove
            ];
            shellHook = ''
              export LD_LIBRARY_PATH="${lib.makeLibraryPath buildInputs}"
              export PATH="$HOME/.cargo/bin:$PATH"
              export ANDROID_NDK_ROOT="${android-sdk}/share/android-sdk/ndk/26.1.10909125"
              export ANDROID_HOME="$PWD/.android_sdk"
              exec fish
            '';
          };
        };
    };
}
