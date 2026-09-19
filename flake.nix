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
            sha256 = "sha256-6pbof85hshggBqgZz41qx0zHVi5LxtYKukEH/8uljVI=";
          };

          muslTargetMap = {
            "x86_64-linux" = "x86_64-unknown-linux-musl";
            "aarch64-linux" = "aarch64-unknown-linux-musl";
          };

          freebsdTargetMap = {
            "x86_64-linux" = "x86_64-unknown-freebsd";
          };

          mkNativeDeps =
            {
              hostPkgs,
              withStatic ? true,
            }:
            let
              hostLib = hostPkgs.lib;
              hostStdenv = hostPkgs.stdenv;

              disableTarget = if withStatic then "shared" else "static";
              enableTarget = if withStatic then "static" else "shared";
              sharedLibs = if withStatic then "OFF" else "ON";
            in
            rec {
              openssl = hostPkgs.openssl.override { static = withStatic; };

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

                  inherit withStatic;
                  withShared = !withStatic;

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
                        ++ (hostLib.optional withStatic "--pkg-config-flags=--static");
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
                      dontDisableStatic = withStatic;
                      postPatch =
                        previousAttrs.postPatch
                        + hostLib.optionalString (withStatic && !hostStdenv.hostPlatform.isStatic) ''
                          substituteInPlace src/interfaces/libpq/Makefile \
                            --replace-fail "all: all-lib libpq-refs-stamp" "all: all-lib"
                          substituteInPlace src/Makefile.shlib \
                            --replace-fail "all-lib: all-shared-lib" "all-lib: all-static-lib" \
                            --replace-fail "install-lib: install-lib-shared" "install-lib: install-lib-static"
                        '';
                      postInstall =
                        if (withStatic || hostStdenv.hostPlatform.isStatic) then
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
              crossSystem ? null,
              withStatic ? true,
              withCoverage ? false,
            }:
            let
              isCross = crossSystem != null;
              hostPkgs =
                if isCross then
                  import nixpkgs {
                    localSystem = {
                      inherit system;
                    };
                    inherit crossSystem;
                  }
                else
                  pkgs;
              hostLib = hostPkgs.lib;
              hostTarget = hostPkgs.stdenv.hostPlatform.config;

              rustTarget = hostPkgs.stdenv.targetPlatform.rust.rustcTarget;
              rustShoutTarget = builtins.replaceStrings [ "-" ] [ "_" ] (hostLib.toUpper rustTarget);
              rustPlatform = hostPkgs.makeRustPlatform {
                cargo = toolchain;
                rustc = toolchain;
              };

              nativeDeps = mkNativeDeps {
                inherit hostPkgs;
                inherit withStatic;
              };

              ccBin = "${hostPkgs.stdenv.cc}/bin/${hostLib.optionalString isCross "${hostTarget}-"}cc";

              cargoNextest = pkgs.cargo-nextest;
              cargoLlvmCov = pkgs.cargo-llvm-cov;
            in
            with hostPkgs;
            pkgs.mkShellNoCC {
              dontAddExtraLibs = true;

              env = rec {
                # cargo
                CARGO_BUILD_TARGET = rustTarget;
                "CARGO_TARGET_${rustShoutTarget}_LINKER" = ccBin;
                "CC_${rustShoutTarget}" = ccBin;

                # native
                PKG_CONFIG_ALL_STATIC = if withStatic then "1" else null;

                OPENSSL_INCLUDE_DIR = "${nativeDeps.openssl.dev}/include";
                OPENSSL_LIB_DIR = "${nativeDeps.openssl.out}/lib";
                OPENSSL_STATIC = if withStatic then "1" else "0";

                PQ_LIB_STATIC = if withStatic then "1" else null;

                # test
                POSTGRES_USER = "postgres";
                POSTGRES_PASSWORD = "postgres";
                POSTGRES_DATABASE = "postgres";
                POSTGRES_PORT = 5432;
                DATABASE_URL = "postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DATABASE}";

                AWS_ACCESS_KEY_ID = "key";
                AWS_SECRET_ACCESS_KEY = "key";
                AWS_REGION = "us-east-1";
                AWS_PORT = 9090;
                AWS_USE_PATH_STYLE_ENDPOINT = "true";
                AWS_ENDPOINT_URL = "http://localhost:${AWS_PORT}";
              };

              packages = [
                toolchain
                cargoNextest

                pkg-config

                # native
                pkgs.stdenv.cc
                stdenv.cc
                pkgs.llvmPackages.libclang.lib
                (rustPlatform.bindgenHook.override { clang = pkgs.clang; })

                # test
                pkgs.podman
                pkgs.podman-compose
              ]
              ++ (hostLib.optional withCoverage cargoLlvmCov)
              ++ (hostLib.attrValues nativeDeps)
              ++ hostLib.optional stdenv.hostPlatform.isLinux autoPatchelfHook
              ++
                hostLib.optional stdenv.hostPlatform.isDarwin
                  (if withStatic then hostPkgs.pkgsStatic else hostPkgs).darwin.libiconv;
            };
        in
        {
          devShells = {
            default = mkDevShell { };
          }
          // (lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            gnu = mkDevShell { };
            musl = mkDevShell {
              crossSystem = {
                config = muslTargetMap.${system};
                isStatic = true;
              };
            };
            freebsd = mkDevShell {
              crossSystem = {
                config = freebsdTargetMap.${system};
              };
            };
            coverage = mkDevShell { withCoverage = true; };
          });
        };
    };
}
