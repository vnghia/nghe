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

  nixConfig = {
    extra-substituters = "https://nghe.cachix.org";
    extra-trusted-public-keys = "nghe.cachix.org-1:H757ngFI7o6unzi/uiXKN6fhfQjCjrEI80hZDWr0Mrs=";
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
        "aarch64-darwin"
      ];

      perSystem =
        {
          system,
          pkgs,
          lib,
          ...
        }:
        let
          toolchain = inputs.fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-qaiVSNjXbvb8Jd+Zo0y59MrGs8KQwCQ/QwSm4Gn4Lxs=";
          };

          rustTargetMap = {
            "x86_64-linux" = "x86_64-unknown-linux-gnu";
            "aarch64-linux" = "aarch64-unknown-linux-gnu";
            "musl64" = "x86_64-unknown-linux-musl";
            "aarch64-multiplatform-musl" = "aarch64-unknown-linux-musl";
            "aarch64-darwin" = "aarch64-apple-darwin";
          };

          muslTargetMap = {
            "x86_64-linux" = "musl64";
            "aarch64-linux" = "aarch64-multiplatform-musl";
          };

          mkNativeDeps =
            {
              hostPkgs,
              static ? true,
            }:
            let
              hostLib = hostPkgs.lib;
              hostStdenv = hostPkgs.stdenv;

              disableTarget = if static then "shared" else "static";
              enableTarget = if static then "static" else "shared";
              sharedLibs = if static then "OFF" else "ON";
            in
            rec {
              openssl = hostPkgs.openssl.override { inherit static; };

              lame =
                (hostPkgs.lame.override {
                  frontendSupport = false;
                }).overrideAttrs
                  (
                    finalAttrs: previousAttrs: {
                      configureFlags = previousAttrs.configureFlags ++ [
                        "--disable-${disableTarget}"
                        "--enable-${enableTarget}"
                      ];
                    }
                  );

              libopus = hostPkgs.libopus.overrideAttrs (
                finalAttrs: previousAttrs: {
                  mesonBuildType = "release";
                  mesonFlags = previousAttrs.mesonFlags ++ [
                    "-Ddefault_library=${enableTarget}"
                    "-Ddefault_both_libraries=${enableTarget}"
                  ];
                }
              );

              soxr = hostPkgs.soxr.overrideAttrs (
                finalAttrs: previousAttrs: {
                  cmakeFlags = previousAttrs.cmakeFlags ++ [
                    "-DWITH_OPENMP=OFF"
                    "-DBUILD_SHARED_LIBS=${sharedLibs}"
                  ];
                }
              );

              libogg = hostPkgs.libogg.overrideAttrs (
                finalAttrs: previousAttrs: {
                  cmakeFlags = previousAttrs.cmakeFlags ++ [
                    "-DBUILD_SHARED_LIBS=${sharedLibs}"
                  ];
                }
              );

              libvorbis = (hostPkgs.libvorbis.override { inherit libogg; }).overrideAttrs (
                finalAttrs: previousAttrs: {
                  configureFlags = [
                    "--disable-${disableTarget}"
                    "--enable-${enableTarget}"
                  ];
                }
              );

              ffmpeg =
                (hostPkgs.ffmpeg.override {
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
                  withStatic = static;
                  withShared = !static;
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
                  withStripping = true;

                  inherit lame;
                  inherit libopus;
                  inherit soxr;
                  inherit libvorbis;
                }).overrideAttrs
                  (
                    finalAttrs: previousAttrs: {
                      configureFlags =
                        previousAttrs.configureFlags
                        ++ [
                          "--enable-openssl"
                          "--extra-libs=-lm"
                        ]
                        ++ (hostLib.optional static "--pkg-config-flags=--static");
                      buildInputs = previousAttrs.buildInputs ++ [
                        openssl
                      ];
                    }
                  );

              # test
              libpq =
                (hostPkgs.libpq.override {
                  curlSupport = false;
                  gssSupport = false;
                  nlsSupport = false;

                  inherit openssl;
                }).overrideAttrs
                  (
                    finalAttrs: previousAttrs: {
                      dontDisableStatic = static;
                      postPatch =
                        previousAttrs.postPatch
                        + hostLib.optionalString (static && !hostStdenv.hostPlatform.isStatic) ''
                          substituteInPlace src/interfaces/libpq/Makefile \
                            --replace-fail "all: all-lib libpq-refs-stamp" "all: all-lib"
                          substituteInPlace src/Makefile.shlib \
                            --replace-fail "all-lib: all-shared-lib" "all-lib: all-static-lib" \
                            --replace-fail "install-lib: install-lib-shared" "install-lib: install-lib-static"
                        '';
                      postInstall =
                        if (static || hostStdenv.hostPlatform.isStatic) then
                          ''
                            touch $out/empty
                            substituteInPlace $out/lib/pkgconfig/libpq.pc \
                              --replace-fail "$out" "$dev"
                          ''
                        else
                          previousAttrs.postInstall;
                    }
                  );
            };

          mkDevShell =
            {
              target ? system,
              static ? true,
              coverage ? false,
            }:
            let
              isCross = target != system;
              hostPkgs =
                if isCross then
                  import nixpkgs {
                    localSystem = {
                      inherit system;
                    };
                    crossSystem = {
                      system = pkgs.pkgsCross.${target}.stdenv.hostPlatform.config;
                    };
                  }
                else
                  pkgs;
              hostLib = hostPkgs.lib;

              rustTarget = rustTargetMap.${target};
              rustShoutTarget = builtins.replaceStrings [ "-" ] [ "_" ] (hostLib.toUpper rustTarget);
              rustPlatform = hostPkgs.makeRustPlatform {
                cargo = toolchain;
                rustc = toolchain;
              };

              nativeDeps = mkNativeDeps {
                inherit hostPkgs;
                inherit static;
              };

              ccBin = "${hostPkgs.stdenv.cc}/bin/${hostLib.optionalString isCross "${rustTarget}-"}cc";

              cargoNextest = hostPkgs.cargo-nextest.override { inherit rustPlatform; };
              cargoTarpaulin = hostPkgs.cargo-tarpaulin.override {
                inherit rustPlatform;
                openssl = nativeDeps.openssl;
              };
            in
            with hostPkgs;
            pkgs.mkShellNoCC {
              dontAddExtraLibs = true;

              env = {
                # cargo
                CARGO_BUILD_TARGET = rustTarget;
                "CARGO_TARGET_${rustShoutTarget}_LINKER" = ccBin;
                "CC_${rustShoutTarget}" = ccBin;

                # native
                PKG_CONFIG_ALL_STATIC = if static then "1" else null;

                OPENSSL_INCLUDE_DIR = "${nativeDeps.openssl.dev}/include";
                OPENSSL_LIB_DIR = "${nativeDeps.openssl.out}/lib";
                OPENSSL_STATIC = if static then "1" else "0";

                PQ_LIB_STATIC = if static then "1" else null;
              };

              packages = [
                toolchain
                cargoNextest

                pkg-config

                # native
                stdenv.cc
                llvmPackages.libclang.lib
                rustPlatform.bindgenHook
              ]
              ++ (hostLib.optional coverage cargoTarpaulin)
              ++ (hostLib.attrValues nativeDeps)
              ++ hostLib.optional stdenv.hostPlatform.isLinux pkgs.autoPatchelfHook;
            };
        in
        {
          devShells = {
            default = mkDevShell { };
          }
          // (lib.mapAttrs (target: _: mkDevShell { inherit target; }) rustTargetMap)
          // (lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            gnu = mkDevShell { };
            musl = mkDevShell { target = muslTargetMap.${system}; };
            coverage = mkDevShell { coverage = true; };
          });
        };
    };
}
