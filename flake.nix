{
  description = "EventModelRenderer development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            git
            pre-commit
            nodejs_22
            glow
            jq
            # C++ build dependencies for libavoid
            clang
            llvmPackages.libclang
            pkg-config
            # Standard C++ library
            stdenv.cc.cc.lib
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          
          # Required for autocxx to find libclang
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          
          # Set up library paths for the linker
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.stdenv.cc.cc.lib
          ];
        };
      }
    );
}

