# Changelog

## [0.13.0](https://github.com/vnghia/nghe/compare/v0.12.3...v0.13.0) (2026-09-26)


### ⚠ BREAKING CHANGES

* **api:** merge internal endpoint to normal endpoint ([#1121](https://github.com/vnghia/nghe/issues/1121))

### Features

* **api:** merge internal endpoint to normal endpoint ([#1121](https://github.com/vnghia/nghe/issues/1121)) ([7a65aa4](https://github.com/vnghia/nghe/commit/7a65aa421fd789bd2447cb1f337a08ca1cad1f3d))


### Bug Fixes

* **backend/extract:** remove redudant clone ([#1124](https://github.com/vnghia/nghe/issues/1124)) ([17ccf34](https://github.com/vnghia/nghe/commit/17ccf3459d4329dfaeb09b2413197129c0d79642))
* **deps:** migrate to darling for proc-macro ([#1117](https://github.com/vnghia/nghe/issues/1117)) ([3ca42b8](https://github.com/vnghia/nghe/commit/3ca42b8841e7a70b8e1db4fbe0c51c5ddce18d94))
* **deps:** update rust crate lofty to 0.25.0 ([#1063](https://github.com/vnghia/nghe/issues/1063)) ([d9e6ae7](https://github.com/vnghia/nghe/commit/d9e6ae7a17727ef23c1d3933904c8d62deb010d5))
* **deps:** update rust crate tower-http to 0.7.0 ([#1075](https://github.com/vnghia/nghe/issues/1075)) ([3a599f5](https://github.com/vnghia/nghe/commit/3a599f55cac4fd7460b467a8a4d85d6ffc698fbe))

## [0.12.3](https://github.com/vnghia/nghe/compare/v0.12.2...v0.12.3) (2026-09-20)


### Bug Fixes

* **ci:** upload-release need git ([#1114](https://github.com/vnghia/nghe/issues/1114)) ([d5a9a31](https://github.com/vnghia/nghe/commit/d5a9a313485445d58441661954cefa3e940b64a1))

## [0.12.2](https://github.com/vnghia/nghe/compare/v0.12.1...v0.12.2) (2026-09-20)


### Bug Fixes

* **ci:** macos build artifact name ([#1112](https://github.com/vnghia/nghe/issues/1112)) ([7dcae03](https://github.com/vnghia/nghe/commit/7dcae03bf1b1cc2b90ae14ca7c084c96fa562e2f))

## [0.12.1](https://github.com/vnghia/nghe/compare/v0.12.0...v0.12.1) (2026-09-20)


### Bug Fixes

* **ci:** build with correct profile ([#1110](https://github.com/vnghia/nghe/issues/1110)) ([8c4ee2a](https://github.com/vnghia/nghe/commit/8c4ee2a91b2cd92efcb096335f94ab23d4954930))

## [0.12.0](https://github.com/vnghia/nghe/compare/v0.11.0...v0.12.0) (2026-09-20)


### Features

* **ci:** switch to nix for native dependencies ([#1064](https://github.com/vnghia/nghe/issues/1064)) ([d61fb78](https://github.com/vnghia/nghe/commit/d61fb787a21c41a58dbe830fa4db5cd21ecb3d3f))
* **ci:** use release-plz ([#1066](https://github.com/vnghia/nghe/issues/1066)) ([0d0638b](https://github.com/vnghia/nghe/commit/0d0638b1999c79d4cfbe52650a0f0bafe98ad1cf))


### Bug Fixes

* **ci:** correct package name ([09be13a](https://github.com/vnghia/nghe/commit/09be13ad7eb01cad1887620d2bd37ccb2ad0142f))
* **ci:** finalize release pipeline ([#1107](https://github.com/vnghia/nghe/issues/1107)) ([a3e4174](https://github.com/vnghia/nghe/commit/a3e41746652fd0c21a9f20c6a156365523e8507c))
* **ci:** fix ccbin path ([#1106](https://github.com/vnghia/nghe/issues/1106)) ([148010c](https://github.com/vnghia/nghe/commit/148010c2977a1a43e4c78e12bd2cab7126944717))
* **ci:** no manual rustc target ([#1069](https://github.com/vnghia/nghe/issues/1069)) ([690867b](https://github.com/vnghia/nghe/commit/690867bff62d0fecef86f3ce88b2649ac28e7daa))
* **ci:** update release-please manifest and config ([fce79d2](https://github.com/vnghia/nghe/commit/fce79d20b5b3d5d02a72a52f0a99041f80fe5a88))
* **ci:** use release-please ([2cc299e](https://github.com/vnghia/nghe/commit/2cc299e7b9f7719ac90be4cf7dfabbc323988ee8))
* **ci:** use repository owner instead of actor ([#1020](https://github.com/vnghia/nghe/issues/1020)) ([3012916](https://github.com/vnghia/nghe/commit/301291665d1b1c89f7ce99779485dadef30ab1bb))
* **ci:** use version group to release ([#1067](https://github.com/vnghia/nghe/issues/1067)) ([aeb9192](https://github.com/vnghia/nghe/commit/aeb919237393862bd0081ee7cfdeaa04a37e70ad))
* **deps:** update rust crate diesel-async to 0.8.0 ([#1056](https://github.com/vnghia/nghe/issues/1056)) ([089854f](https://github.com/vnghia/nghe/commit/089854fba1bf1b66721fae6a70601b0eac7811df))
* **deps:** update rust crate diesel-async to 0.9.0 ([#1074](https://github.com/vnghia/nghe/issues/1074)) ([98005b7](https://github.com/vnghia/nghe/commit/98005b784067312b59705ef7dbe7ac58d84e2211))
* **deps:** update rust crate lofty to 0.23.0 ([#986](https://github.com/vnghia/nghe/issues/986)) ([a98d90e](https://github.com/vnghia/nghe/commit/a98d90e5df51c044f042caab2ba956d1ce84c5c5))
* **deps:** update rust crate rspotify to 0.16.0 ([#1057](https://github.com/vnghia/nghe/issues/1057)) ([cea6007](https://github.com/vnghia/nghe/commit/cea600787522d13c57f8f773451a181298cbcce0))
* **lint:** fix lint ([#1065](https://github.com/vnghia/nghe/issues/1065)) ([24ba296](https://github.com/vnghia/nghe/commit/24ba296f22ef87cfbf21f84e353b3c87ce87c29e))
* **nix:** more freely cross configure ([#1070](https://github.com/vnghia/nghe/issues/1070)) ([2a2a11a](https://github.com/vnghia/nghe/commit/2a2a11a83520fe54e47ff6d40edb9a0e26cdbac5))
