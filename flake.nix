{
  description = "Generate mazes";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    android-nixpkgs.url = "github:tadfisher/android-nixpkgs";
  };

  outputs = inputs@{ flake-parts, android-nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { pkgs, lib, system, ... }:
        let
          android-sdk = android-nixpkgs.sdk.${system} (sdkPkgs: with sdkPkgs; [
            cmdline-tools-latest
            build-tools-34-0-0
            platform-tools
            platforms-android-34
            emulator
            ndk-26-1-10909125
          ]);

        in
        {
          devShells.default = pkgs.mkShell rec {
            buildInputs = with pkgs; [
              rustfmt
              pkg-config
              gdb
              openssl
              libGL
              rust-analyzer

              wayland
              udev
              vulkan-loader
              alsa-lib
              vulkan-loader
              libxkbcommon
              # libcxx
              stdenv.cc.cc.lib

              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr

              lld_18
              rustup
              nodejs_22
              xsel

              android-sdk
              aapt # still needed?
              jdk # remove
            ];
            shellHook = ''
              exec fish
              export PATH="$PATH:${android-sdk}/share/android-sdk/build-tools/34.0.0"
            '';
            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
            ANDROID_NDK_ROOT = "${android-sdk}/share/android-sdk/ndk/26.1.10909125";
          };
        };
    };
}
