{
  description = "Window-aware shell hook daemon";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "focuskeyd";
            version = "0.1.0";

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./LICENSE
                ./examples
                ./src
              ];
            };

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Window-aware IME policy daemon";
              license = lib.licenses.mit;
              mainProgram = "focuskeyd";
              platforms = lib.platforms.linux;
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          ciDeps = with pkgs; [
            cargo
            rustc
          ];
          devDeps = with pkgs; [
            rust-analyzer
            rustfmt
          ];
        in
        {
          default = pkgs.mkShell {
            buildInputs = ciDeps ++ devDeps;
            shellHook = /* bash */ ''
              export RUST_BACKTRACE=1
              export RUST_LOG=focuskeyd=debug
            '';
          };
          ci = pkgs.mkShell {
            buildInputs = ciDeps;
          };
        }
      );
    };
}
