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
        rec {
          bgsvg = pkgs.rustPlatform.buildRustPackage {
            pname = "bgsvg";
            version = "0.1.0";
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
              ];
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.protobuf ];
            PROTOC = "${pkgs.protobuf}/bin/protoc";
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
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
        };
      });
    };
}
