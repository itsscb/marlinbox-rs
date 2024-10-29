{
  description = "Example Rust development environment for Zero to Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            # trunk
            clippy
            tailwindcss

            # dioxus-cli
            # wasm-bindgen-cli

            # cargo-shuttle
            cargo-edit
            cargo-binstall
            bacon

            alsa-lib.dev

            openssl
            pkg-config
          ];

          shellHook = ''
            export PATH=${rustToolchain}/bin:$PATH
            export RUSTC_VERSION=$(rustc --version)
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export OPENSSL_DIR="${pkgs.openssl.dev}"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
          '';

          packages = pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);
        };
      }
    );
}
