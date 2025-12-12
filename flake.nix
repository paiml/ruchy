{
  description = "Ruchy - A programming language with Python syntax and Rust performance";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pin Rust version for reproducibility
        rustToolchain = pkgs.rust-bin.stable."1.83.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common arguments for crane builds
        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;

          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          # Reproducibility: Fixed RUSTFLAGS
          RUSTFLAGS = "-C target-cpu=native";
        };

        # Build just the cargo dependencies
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate
        ruchy = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          # Reproducibility: Fixed build timestamp
          SOURCE_DATE_EPOCH = "1704067200"; # 2024-01-01 00:00:00 UTC
        });

      in {
        checks = {
          inherit ruchy;

          ruchy-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });

          ruchy-fmt = craneLib.cargoFmt {
            src = craneLib.path ./.;
          };

          ruchy-test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        packages = {
          default = ruchy;
          inherit ruchy;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = ruchy;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            # Rust tooling
            rustToolchain
            cargo-watch
            cargo-audit
            cargo-outdated
            cargo-criterion
            cargo-mutants
            cargo-llvm-cov

            # Build tools
            pkg-config
            openssl

            # Development tools
            just
            jq

            # Benchmarking
            hyperfine

            # WASM
            wasm-pack
            wasm-bindgen-cli
          ];

          # Reproducibility: Set environment variables
          RUST_BACKTRACE = "1";
          RUST_LOG = "info";

          shellHook = ''
            echo "Ruchy development environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
          '';
        };
      }
    );
}
