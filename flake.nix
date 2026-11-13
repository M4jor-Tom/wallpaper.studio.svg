{
  description = "svg_builder — animated SVG background generator (mood + resolution matching)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAll (
        pkgs:
        let
          # only what the build reads; ./tests holds the goldens `cargo test` checks
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./build.rs
              ./parameters.proto
              ./askama.toml
              ./src
              ./templates
              ./tests
              ./crates/bgsvg-wasm/Cargo.toml
              ./crates/bgsvg-wasm/src
            ];
          };
        in
        rec {
          bgsvg = pkgs.rustPlatform.buildRustPackage {
            pname = "bgsvg";
            version = "0.1.0";
            inherit src;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.protobuf ];
            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };
          bgsvg-wasm = pkgs.rustPlatform.buildRustPackage {
            pname = "bgsvg-wasm";
            version = "0.1.0";
            inherit src;
            cargoLock.lockFile = ./Cargo.lock;
            # wasm-bindgen-cli must match Cargo.lock's wasm-bindgen exactly. On a "schema version" mismatch:
            # `cargo update -p wasm-bindgen --precise <cli version>` (and js-sys). lld: nixpkgs ships no rust-lld for wasm32.
            nativeBuildInputs = [ pkgs.protobuf pkgs.wasm-bindgen-cli pkgs.lld ];
            PROTOC = "${pkgs.protobuf}/bin/protoc";
            buildPhase = ''
              cargo build --release --target wasm32-unknown-unknown -p bgsvg-wasm
            '';
            # web for bundlers, nodejs for tests/wasm.mjs
            installPhase = ''
              for t in web nodejs; do
                wasm-bindgen target/wasm32-unknown-unknown/release/bgsvg_wasm.wasm \
                  --out-dir $out/$t --target $t
              done
            '';
            doCheck = false;
          };
          default = bgsvg;
        }
      );

      apps = forAll (pkgs: rec {
        bgsvg = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.bgsvg}/bin/bgsvg";
        };
        default = bgsvg;
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.clippy
            pkgs.protobuf
            pkgs.lld
            pkgs.wasm-bindgen-cli
            # only for tests/wasm.mjs, via tests/wasm.rs
            pkgs.nodejs
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
        };
      });
    };
}
