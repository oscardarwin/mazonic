{
  description = "Generate mazes";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { pkgs, lib, ... }: {
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            rustc
            cargo
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
            libcxx

            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr

            lld_18
            rustup
            nodejs_22
            wasm-bindgen-cli
            xsel
          ];
          shellHook = ''
            exec fish
          '';
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      };
    };
}
