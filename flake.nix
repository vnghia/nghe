{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    allow-import-from-derivation = true;
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        {
          system,
          pkgs,
          ...
        }:
        let
          rustToolchain = inputs.fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-qaiVSNjXbvb8Jd+Zo0y59MrGs8KQwCQ/QwSm4Gn4Lxs=";
          };
        in
        {
          devShells.default =
            with pkgs;
            mkShell {
              env = {
                LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
                LD_LIBRARY_PATH = lib.makeLibraryPath [
                  postgresql.lib
                ];
              };

              packages = [
                rustToolchain

                # ffmpeg
                zip
                unzip
                pkg-config
                python3
                nasm

                clang
                llvmPackages.libclang
                stdenv.cc

                # test
                postgresql.lib
              ];
            };
        };
    };
}
