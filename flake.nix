{
  description = "Sample devShell flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    {
      devShells =
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-darwin"
          ]
          (
            system:
            let
              pkgs = import nixpkgs {
                inherit system;
              };
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
                shellHook = ''

                '';
              };
            }
          );
    };
}
