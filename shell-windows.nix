{
  pkgs' ? import <nixpkgs> { },
  pkgs ? pkgs'.pkgsCross.mingw32,
}:

pkgs.mkShell {
  name = "shell-rust";
  nativeBuildInputs = [
    (pkgs'.rust-bin.stable.latest.minimal.override {
      targets = [ pkgs.targetPlatform.rust.cargoShortTarget ];
    })

  ];
  buildInputs = [ pkgs.windows.pthreads ];

  "CARGO_TARGET_${pkgs.targetPlatform.rust.cargoEnvVarTarget}_LINKER" = "${pkgs.stdenv.cc.targetPrefix}cc";
}
