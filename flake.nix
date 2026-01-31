{
  description = "Mycelix Supply Chain ERP - Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            cargo
            cargo-watch
            cargo-edit

            # Build dependencies
            pkg-config
            openssl
            openssl.dev

            # PostgreSQL
            postgresql
            postgresql.lib

            # Database tools
            sqlx-cli

            # Development tools
            git
            jq

            # System libraries
            gcc
            cmake
            zlib
            zlib.dev
          ];

          shellHook = ''
            echo "🚀 Mycelix ERP Development Environment"
            echo "======================================"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "PostgreSQL: $(psql --version | head -n1)"
            echo ""
            echo "Environment variables set:"
            echo "  PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
            echo "  LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
            echo ""
            echo "Quick commands:"
            echo "  cargo check --lib      # Check FIN module compiles"
            echo "  cargo build            # Build entire service"
            echo "  cargo test             # Run tests"
            echo "  sqlx migrate run       # Apply database migrations"
            echo ""
          '';

          # Environment variables for compilation
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.postgresql.lib}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.openssl pkgs.postgresql.lib pkgs.zlib ]}";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        };

        # Package the service (for production builds)
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "mycelix-service";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];

          buildInputs = with pkgs; [
            openssl
            postgresql
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      }
    );
}
