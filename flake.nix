{
  description = "Flake for building boa";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.boa-server = pkgs.mkShell {
          name = "boa-server-shell";

          buildInputs = [
            pkgs.rustup
            pkgs.cargo
            pkgs.openssl
            pkgs.pkg-config
            pkgs.git
          ];

          RUST_BACKTRACE = "1";

          shellHook = ''
            echo "You are in the boa-server-shell"
            rustup default stable
          '';
        };

        defaultPackage = self.packages.${system}.boa-server;
      }
    );
}
