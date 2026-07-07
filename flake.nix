{
  description = "A blazing fast Rust text search utility with custom libraries";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        Downs = pkgs.rustPlatform;
      in {
        packages.default = Downs.buildRustPackage {
          pname = "lewrep2";
          version = "2.0.0";
          
          src = pkgs.lib.cleanSource ./.;
          
          # Point directly to the root lockfile we just copied
          cargoLock.lockFile = ./Cargo.lock;
          
          buildAndTestSubdir = "lewrep"; 
        };
      });
}
