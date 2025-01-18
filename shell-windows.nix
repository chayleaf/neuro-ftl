{
  pkgs' ? import <nixpkgs> { },
  pkgs ? pkgs'.pkgsCross.mingw32,
}:

/*let
  cc = (pkgs.stdenv.cc.override ({
      extraBuildCommands = ''
          printf '%s' ' -L${pkgs.windows.mcfgthreads}/lib' >> $out/nix-support/cc-ldflags
          printf '%s' ' -I${pkgs.windows.mcfgthreads.dev}/include' >> $out/nix-support/cc-cflags
          printf '%s' ' -L${pkgs.windows.pthreads}/lib' >> $out/nix-support/cc-ldflags
          printf '%s' ' -I${pkgs.windows.pthreads}/include' >> $out/nix-support/cc-cflags
      '';
    }));
in*/
(pkgs.mkShell/*.override ({ stdenv = pkgs.clangStdenv; })*/) {
  name = "shell-rust";
  nativeBuildInputs = [
    (pkgs'.rust-bin.stable.latest.minimal.override {
      targets = [ pkgs.targetPlatform.rust.cargoShortTarget ];
    })
    # cc
  ];
  buildInputs = [
    #pkgs.windows.pthreads
    #pkgs.windows.mcfgthreads
    
  ];

  "CARGO_TARGET_${pkgs.targetPlatform.rust.cargoEnvVarTarget}_LINKER" = "${pkgs.stdenv.cc.targetPrefix}cc";
  "CARGO_TARGET_${pkgs.targetPlatform.rust.cargoEnvVarTarget}_RUSTFLAGS" = "-L native=${pkgs.windows.pthreads}/lib -L native=${pkgs.windows.mcfgthreads}/lib -l mcgthreads";
}
