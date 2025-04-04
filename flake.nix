{
  inputs = {
    utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              just
              bash
              parallel
              rustup
              gcc
              pnpm
              nodejs
              wasm-pack
              zip
              pandoc
            ];
          };
      }
    );
}
