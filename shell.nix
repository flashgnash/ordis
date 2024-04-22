{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    sqlite

    openssl.dev
  ];

	PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

  RUST_BACKTRACE = "full";
}
