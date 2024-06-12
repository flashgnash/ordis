{
  description = "Ordis discord bot";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=23.11";

  };

  outputs = { self, nixpkgs }: 
  let 
    system="x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
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
        ];

        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

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
