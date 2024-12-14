{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [ 
      rustc
      rustup
      cargo
      sqlite
      dos2unix
    ];
}
