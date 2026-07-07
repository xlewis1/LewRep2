{
  description = "lewrep2: a parallel search utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        rustPlatform = pkgs.rustPlatform;
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "lewrep2";
          version = "2.1.0";

          src = pkgs.lib.cleanSourceWith {
            src = ../.;
            filter = path: type:
              let baseName = baseNameOf path; in
              baseName != "target" && baseName != ".git";
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          prePatch = ''
            cp ${./Cargo.lock} Cargo.lock
          '';

          buildAndTestSubdir = "lewrep";

          nativeBuildInputs = [ pkgs.pkg-config ];

          meta = with pkgs.lib; {
            description = "A parallel search utility";
            homepage = "https://github.com/xlewis1/LewRep2";
            license = licenses.mit;
            mainProgram = "lewrep2";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.rustfmt
          ];
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/lewrep2";
        };
      });
}
