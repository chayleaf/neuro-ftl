{
  pkgs ? import <nixpkgs> { system = "i686-linux"; },
}:

pkgs.mkShell {
  name = "shell-rust";
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
  ];
}
