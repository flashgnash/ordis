{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    sqlite
  ];

  RUST_BACKTRACE = "full";
}
