{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  name = "dev-env";

  buildInputs = with pkgs; [
    openssl
    gcc
    rustc
    cargo
    cargo-watch
    nodejs-18_x
    pnpm
    tailwindcss
    biome
  ];

  shellHook = ''
    cd web/ 
    pnpm run dev &
    cd ../iris/
    cargo watch -x run &
  '';
}

