{
  description = "Zmem Linux memory monitor";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        with pkgs;
        rec {
          zmem = rustPlatform.buildRustPackage {
            pname = "zmem";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            meta = {
              description = "Linux memory monitor with detailed virtual memory information";
              homepage = "https://github.com/xeome/zmem";
              license = lib.licenses.gpl3Only;
              mainProgram = "zmem";
              platforms = lib.platforms.linux;
            };
          };
          default = zmem;
        }
      );
    };
}
