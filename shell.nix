{
  pkgs ? import <nixpkgs> { system = "i686-linux"; },
}:

pkgs.mkShell {
  name = "shell-rust";
  nativeBuildInputs = with pkgs; [ cargo rustc ];
  # buildInputs = with pkgs; [ openssl ];
  # LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";
}
