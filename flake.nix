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
    let
      perSystem = flake-utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          new-migration = pkgs.writeShellScriptBin "new-migration" "diesel migration generate $1";
          redo-migration = pkgs.writeShellScriptBin "redo-migration" "diesel migration redo";
          run-migration = pkgs.writeShellScriptBin "run-migration" "diesel migration run";
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
          rustPkgs = import ./Cargo.nix {
            inherit pkgs;
            defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              audiopus_sys = attrs: {
                nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [
                  pkgs.cmake
                  pkgs.pkg-config
                ];
                buildInputs = (attrs.buildInputs or [ ]) ++ [
                  pkgs.libopus
                ];
              };
              ordis = attrs: {
                nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [
                  pkgs.pkg-config
                ];
                buildInputs = (attrs.buildInputs or [ ]) ++ [
                  pkgs.sqlite
                  pkgs.openssl.dev
                ];
              };
            };
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
              cachix
              crate2nix
              diesel-cli
              pkg-config
              openssl.dev
              lldb
              new-migration
              redo-migration
              run-migration
              gen-up
              gen-down
              (pkgs.writeShellScriptBin "run" "cargo run")
              cmake
              libopus
              yt-dlp
              postgresql
              postgrest
            ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OUT_DIR = "./src/db";
            RUST_BACKTRACE = "full";
          };
          packages.default = rustPkgs.workspaceMembers."ordis".build.overrideAttrs (old: {
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OUT_DIR = "./src/db";
            RUST_BACKTRACE = "full";
            nativeBuildInputs = old.nativeBuildInputs or [ ] ++ [
              pkgs.cmake
              pkgs.pkg-config
            ];
            buildInputs = old.buildInputs or [ ] ++ [
              pkgs.sqlite
              pkgs.openssl.dev
              pkgs.libopus
            ];
          });
        }
      );
    in
    perSystem
    // {
      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.services.ordis;
        in
        {
          options.services.ordis = {
            enable = lib.mkEnableOption "Ordis discord bot";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The Ordis package to use.";
            };

            envFile = lib.mkOption {
              type = lib.types.path;
              default = "${cfg.dataDir}/.env";
              description = "Path to the .env file containing secrets (e.g. Discord token).";
              example = "/run/secrets/ordis.env";
            };

            dataDir = lib.mkOption {
              type = lib.types.str;
              default = "/var/lib/ordis";
              description = "Working directory for Ordis (SQLite DB will live here).";
            };

            user = lib.mkOption {
              type = lib.types.str;
              default = "ordis";
              description = "User to run Ordis as.";
            };

            group = lib.mkOption {
              type = lib.types.str;
              default = "ordis";
              description = "Group to run Ordis as.";
            };
          };

          config = lib.mkIf cfg.enable {
            users.users.${cfg.user} = {
              isSystemUser = true;
              group = cfg.group;
              home = cfg.dataDir;
              createHome = true;
            };
            users.groups.${cfg.group} = { };

            systemd.tmpfiles.rules = [
              "d ${cfg.dataDir} 0750 ${cfg.user} ${cfg.group} -"
              "f ${cfg.envFile} 0600 ${cfg.user} ${cfg.group} -"
            ];

            systemd.services.ordis = {
              description = "Ordis Discord Bot";
              wantedBy = [ "multi-user.target" ];
              after = [ "network-online.target" ];
              wants = [ "network-online.target" ];

              serviceConfig = {
                Type = "simple";
                User = cfg.user;
                Group = cfg.group;
                WorkingDirectory = cfg.dataDir;
                EnvironmentFile = cfg.envFile;
                ExecStart = "${cfg.package}/bin/ordis";
                Restart = "on-failure";
                RestartSec = 10;

                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = [ cfg.dataDir ];
                PrivateTmp = true;
                ProtectKernelTunables = true;
                ProtectControlGroups = true;
                RestrictSUIDSGID = true;
              };
            };
          };
        };
    };
}
