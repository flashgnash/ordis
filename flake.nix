{
  description = "Ordis discord bot";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  };

  outputs = { self, nixpkgs }: 
  let 
    system="x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    new-migration = pkgs.writeShellScriptBin "new-migration" (''diesel migration generate $1 '');

    redo-migration = pkgs.writeShellScriptBin "redo-migration" (''diesel migration redo'');

    run-migration = pkgs.writeShellScriptBin "run-migration" (''diesel migration run'');

  in
  {
    
    devShells.${system}.default = 
      pkgs.mkShell {
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

          new-migration
          redo-migration
          run-migration
        ];


        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        OUT_DIR = "./src/db";
        RUST_BACKTRACE = "full";
    };

    packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
      pname = "Ordis";
      version = "0.0.1";
      src = ./.;
  
      cargoLock = {
        lockFile = ./Cargo.lock;
      };

      nativeBuildInputs = with pkgs; [ pkg-config openssl.dev sqlite];
      PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

      # buildPhase = ''
      #   cargo build
      # '';
      #
      # installPhase = ''
      #   mkdir -p $out/bin
      #   cp target/debug/ordis $out/bin
      # '';

    };
  };
}
