{
  description = "Ordis discord bot";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        new-migration = pkgs.writeShellScriptBin "new-migration" (''diesel migration generate $1 '');

        redo-migration = pkgs.writeShellScriptBin "redo-migration" (''diesel migration redo'');

        run-migration = pkgs.writeShellScriptBin "run-migration" (''diesel migration run'');

        build-and-debug = pkgs.writeShellScriptBin "build-and-debug" (''cargo build && lldb'');
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            rust-analyzer
            clippy
            sqlite
            sqlite-web

            diesel-cli
            openssl.dev

            lldb

            new-migration
            redo-migration
            run-migration

            build-and-debug
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OUT_DIR = "./src/db";
          RUST_BACKTRACE = "full";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "Ordis";
          version = "0.0.1";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl.dev
            sqlite
          ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      }
    );
}
