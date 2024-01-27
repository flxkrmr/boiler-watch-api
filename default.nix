{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [ 
      rustc
      cargo
      sqlite
      dos2unix
    ];
}
