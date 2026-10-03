{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
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
              build = rec {
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
                    withPixelutils = false;
                    withPic = true;
                    withThumb = false;

                    buildFfmpeg = false;
                    buildFfplay = false;
                    buildFfprobe = false;
                    buildQtFaststart = false;
                    buildAvcodec = true;
                    buildAvdevice = false;
                    buildAvfilter = true;
                    buildAvformat = true;
                    buildAvutil = true;
                    buildSwresample = true;
                    buildSwscale = false;

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
                        doCheck = false;
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
              };

              check = {
                libpq =
                  (hostPkgs.libpq.override {
                    curlSupport = false;
                    gssSupport = false;
                    nlsSupport = false;

                    openssl = build.openssl;
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
            };

          mkDevShellAndPackage =
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
              hostStdenv = hostPkgs.stdenv;
              hostTarget = hostStdenv.hostPlatform.config;

              canExecute = hostStdenv.buildPlatform.canExecute hostStdenv.hostPlatform;

              rustBin = inputs.rust-overlay.lib.mkRustBin { } hostPkgs.buildPackages;
              toolchain = rustBin.fromRustupToolchainFile ./rust-toolchain.toml;

              rustTarget = hostStdenv.targetPlatform.rust.rustcTarget;
              rustShoutTarget = builtins.replaceStrings [ "-" ] [ "_" ] (hostLib.toUpper rustTarget);
              rustPlatform = hostPkgs.makeRustPlatform {
                cargo = toolchain;
                rustc = toolchain;
              };

              nativeDeps = mkNativeDeps {
                inherit hostPkgs;
                inherit withStatic;
              };

              ccBin = "${hostStdenv.cc}/bin/${hostLib.optionalString isCross "${hostTarget}-"}cc";
              buildCcBin = "${pkgs.stdenv.cc}/bin/cc";

              cargoExpand = pkgs.cargo-expand;
              cargoNextest = pkgs.cargo-nextest;
              cargoLlvmCov = pkgs.cargo-llvm-cov;

              nativeBuildInputs = with hostPkgs; [
                # rust
                toolchain
                cargoExpand
                cargoNextest

                # native
                pkg-config
                stdenv.cc
                pkgs.llvmPackages.libclang.lib
                (rustPlatform.bindgenHook.override { clang = pkgs.clang; })
              ];

              buildInputs =
                hostLib.attrValues nativeDeps.build
                ++ (hostLib.optional hostStdenv.hostPlatform.isDarwin
                  (if withStatic then hostPkgs.pkgsStatic else hostPkgs).darwin.libiconv
                );

              nativeCheckInputs = with pkgs; [
                postgresql
                seaweedfs
                lsof
              ];

              checkInputs = hostLib.attrValues nativeDeps.check;

              env = {
                build = {
                  # stdenv
                  CC = ccBin;
                  "CC_${rustShoutTarget}" = ccBin;
                  CARGO_BUILD_TARGET = rustTarget;
                  "CARGO_TARGET_${rustShoutTarget}_LINKER" = ccBin;

                  # native
                  PKG_CONFIG_ALL_STATIC = if withStatic then "1" else null;
                  OPENSSL_STATIC = if withStatic then "1" else "0";
                  PQ_LIB_STATIC = if withStatic then "1" else null;

                  # git
                  GIT_COMMIT_HASH_SHORT = builtins.substring 0 7 (
                    self.rev or (hostLib.removeSuffix "-dirty" self.dirtyRev)
                  );
                }
                // (hostLib.optionalAttrs (isCross && !canExecute) {
                  HOST_CC = buildCcBin;
                });

                check = rec {
                  RUST_LOG = "nghe=trace";

                  POSTGRES_USER = "postgres";
                  POSTGRES_PASSWORD = "postgres";
                  POSTGRES_DATABASE = "postgres";
                  POSTGRES_PORT = "5432";
                  DATABASE_URL = "postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DATABASE}";

                  AWS_ACCESS_KEY_ID = "key-id";
                  AWS_SECRET_ACCESS_KEY = "access-key";
                  AWS_REGION = "us-east-1";
                  AWS_PORT = "9090";
                  AWS_USE_PATH_STYLE_ENDPOINT = "true";
                  AWS_ENDPOINT_URL = "http://localhost:${AWS_PORT}";
                };
              };
              envBuild = env.build;
              envCheck = env.check;
            in
            {
              package = rustPlatform.buildRustPackage (finalAttrs: rec {
                pname = "nghe";
                version = (hostLib.importTOML ./Cargo.toml).workspace.package.version;

                strictDeps = true;

                src = hostLib.sources.sourceByGlobs ./. [
                  "Cargo.toml"
                  "Cargo.lock"
                  ".cargo/config.toml"
                  ".config/nextest.toml"

                  "assets/**"

                  "nghe*/**/Cargo.toml"
                  "nghe*/**/*.rs"
                  "nghe*/**/*.sql"
                ];

                cargoLock.lockFile = ./Cargo.lock;

                doCheck = canExecute;
                useNextest = true;

                cargoTestFlags = [
                  "--frozen"
                  "--profile"
                  "ci"
                  "--workspace"
                  "--exclude"
                  "${pname}-frontend"
                ];

                preCheck = ''
                  export PGDATA="$NIX_BUILD_TOP/postgresql"
                  initdb --username=${envCheck.POSTGRES_USER} \
                    --pwfile=<(echo ${envCheck.POSTGRES_PASSWORD}) \
                    --encoding="UTF-8" \
                    --set "max_connections = 1000" --set "shared_buffers = 2048MB"
                  pg_ctl start -o "-c unix_socket_directories="

                  weed mini -dir=/tmp/weed -s3.port=${envCheck.AWS_PORT} &
                '';

                postCheck = ''
                  pg_ctl stop

                  kill $(lsof -t -i :${envCheck.AWS_PORT})
                '';

                env = envBuild // envCheck // { RUST_BACKTRACE = "1"; };
                inherit nativeBuildInputs;
                inherit buildInputs;
                inherit nativeCheckInputs;
                inherit checkInputs;

                # Use the built-in one after https://github.com/NixOS/nixpkgs/issues/303796.
                # The provided HOST_CC does not work with musl build.
                buildPhase = ''
                  runHook preBuild
                  cargo build --frozen --profile ${finalAttrs.cargoBuildType} --package ${pname}
                  runHook postBuild
                '';
              });

              devShell = pkgs.mkShellNoCC {
                dontAddExtraLibs = true;

                env = envBuild // envCheck;

                packages =
                  buildInputs
                  ++ nativeBuildInputs
                  ++ checkInputs
                  ++ nativeCheckInputs
                  ++ (hostLib.optional withCoverage cargoLlvmCov)
                  ++ (
                    if pkgs.stdenv.hostPlatform.isLinux then
                      [
                        pkgs.docker
                        pkgs.docker-compose
                      ]
                    else
                      [ ]
                  );
              };
            };

          all =
            let
              llvmCrossSystem = {
                useLLVM = true;
                linker = "lld";
              };
              linuxCrossSystem = llvmCrossSystem // {
                config = pkgs.stdenv.hostPlatform.config;
              };
            in
            {
              default = mkDevShellAndPackage {
                crossSystem = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux linuxCrossSystem;
              };
            }
            // (lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
              musl = mkDevShellAndPackage {
                crossSystem = llvmCrossSystem // {
                  config = muslTargetMap.${system};
                  isStatic = true;
                };
              };
              freebsd = mkDevShellAndPackage {
                crossSystem = llvmCrossSystem // {
                  config = freebsdTargetMap.${system};
                };
              };
              coverage = mkDevShellAndPackage {
                crossSystem = linuxCrossSystem;
                withCoverage = true;
              };
            });
        in
        {
          packages = lib.mapAttrs (_: value: value.package) all;
          devShells = lib.mapAttrs (_: value: value.devShell) all;
        };
    };
}
