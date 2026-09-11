{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-26.05";
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
          toolchain = inputs.fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-qaiVSNjXbvb8Jd+Zo0y59MrGs8KQwCQ/QwSm4Gn4Lxs=";
          };

          static = rec {
            openssl = pkgs.openssl.override { static = true; };

            lame =
              (pkgs.lame.override {
                frontendSupport = false;
              }).overrideAttrs
                (
                  finalAttrs: previousAttrs: {
                    configureFlags = previousAttrs.configureFlags ++ [
                      "--disable-shared"
                      "--enable-static"
                    ];
                  }
                );

            libopus = pkgs.libopus.overrideAttrs (
              finalAttrs: previousAttrs: {
                mesonBuildType = "release";
                mesonFlags = previousAttrs.mesonFlags ++ [
                  "-Ddefault_library=static"
                  "-Ddefault_both_libraries=static"
                ];
              }
            );

            soxr = pkgs.soxr.overrideAttrs (
              finalAttrs: previousAttrs: {
                cmakeFlags = previousAttrs.cmakeFlags ++ [
                  "-DBUILD_SHARED_LIBS=OFF"
                  "-DWITH_OPENMP=OFF"
                ];
              }
            );

            ffmpeg =
              (pkgs.ffmpeg.override {
                version = "8.0.3";

                ffmpegVariant = "headless";

                withHeadlessDeps = false;
                withSmallDeps = false;
                withFullDeps = false;

                withMp3lame = true;
                withOpus = true;
                withSoxr = true;
                withVorbis = true;

                withSmallBuild = false;
                withHardcodedTables = true;
                withMultithread = true;
                withNetwork = true;
                withPixelutils = true;
                withStatic = true;
                withShared = false;
                withPic = true;
                withThumb = false;

                buildFfmpeg = false;
                buildFfplay = false;
                buildFfprobe = false;
                buildQtFaststart = false;
                buildAvcodec = true;
                buildAvdevice = true;
                buildAvfilter = true;
                buildAvformat = true;
                buildAvutil = true;
                buildSwresample = true;
                buildSwscale = true;

                withOptimisations = true;

                inherit lame;
                inherit libopus;
                inherit soxr;
              }).overrideAttrs
                (
                  finalAttrs: previousAttrs: {
                    postPatch = previousAttrs.postPatch + ''
                      substituteInPlace configure \
                        --replace-fail \
                          'require libsoxr soxr.h soxr_create -lsoxr' \
                          'require libsoxr soxr.h soxr_create -lsoxr $libm_extralibs'
                    '';
                    configureFlags = previousAttrs.configureFlags ++ [
                      "--enable-openssl"
                    ];
                    buildInputs = previousAttrs.buildInputs ++ [
                      openssl
                    ];
                  }
                );
          };
        in
        {
          packages = static;

          devShells.default =
            with pkgs;
            mkShell {
              env = {
                OPENSSL_INCLUDE_DIR = "${static.openssl.dev}/include";
                OPENSSL_LIB_DIR = "${static.openssl.out}/lib";

                LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

                LD_LIBRARY_PATH = lib.makeLibraryPath [
                  postgresql.lib
                ];
              };

              packages = [
                toolchain
                pkg-config

                # native
                stdenv.cc

                static.openssl

                clang
                llvmPackages.libclang
                static.ffmpeg

                # test
                postgresql.lib
              ];
            };
        };
    };
}
