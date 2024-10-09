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

        build-and-debug = pkgs.writeShellScriptBin "build-and-debug" (''cargo run'');

        gen-up = pkgs.writeShellApplication {
          name = "gen-up";

          text = ''
            chatgpt "
            Please write an up.sql for SQLite based on the following schema.rs: $(cat src/db/schema.rs)
            Assume it is already being run in a transaction (don't add begin and end transaction statements)
            The up.sql should make the following changes:
            $1
            "

          '';

        };

        gen-down = pkgs.writeShellApplication {
          name = "gen-down";

          text = ''

            latest=$(find migrations -type d | sort -r | head -n 1)
            up=$(cat "$latest"/up.sql)
            chatgpt "
            With the context of the current schema ($(cat src/db/schema.rs))
            Please generate a sqlite down.sql for the following up.sql: $up"


          '';
        };

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

            gen-up
            gen-down

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
