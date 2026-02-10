# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.6](https://github.com/vnghia/nghe/releases/tag/nghe_api-v0.10.6) - 2026-02-10

### Added

- *(backend)* add size support for get cover art ([#800](https://github.com/vnghia/nghe/pull/800))
- *(api/backend)* add api for get music folder stats ([#796](https://github.com/vnghia/nghe/pull/796))
- *(backend)* allow immutable for cache control ([#795](https://github.com/vnghia/nghe/pull/795))
- *(frontend)* add create/delete user ([#789](https://github.com/vnghia/nghe/pull/789))
- *(api/backend)* add api for permission ([#772](https://github.com/vnghia/nghe/pull/772))
- *(api)* remove binary encode for frontend ([#768](https://github.com/vnghia/nghe/pull/768))
- *(api/backend)* add api for manage user ([#765](https://github.com/vnghia/nghe/pull/765))
- *(backend)* add an endpoint for healthcheck ([#710](https://github.com/vnghia/nghe/pull/710))

### Fixed

- *(deps)* update rust crate md5 to 0.8.0 ([#907](https://github.com/vnghia/nghe/pull/907))
- *(backend)* increase cache duration for get cover art ([#756](https://github.com/vnghia/nghe/pull/756))

### Other

- bump rust-toolchain to nightly-2026-02-07 ([#948](https://github.com/vnghia/nghe/pull/948))
- *(deps)* update rust crate built to 0.8.0 ([#852](https://github.com/vnghia/nghe/pull/852))
- *(frontend)* add frontend ([#790](https://github.com/vnghia/nghe/pull/790))
- *(backend)* move part of role to user music folder ([#783](https://github.com/vnghia/nghe/pull/783))
- bump dependencies ([#738](https://github.com/vnghia/nghe/pull/738))
- add users wip component ([#675](https://github.com/vnghia/nghe/pull/675))
- rework error boundary ([#663](https://github.com/vnghia/nghe/pull/663))
- add applications shell ([#657](https://github.com/vnghia/nghe/pull/657))
- add login components ([#647](https://github.com/vnghia/nghe/pull/647))
- add setup components ([#644](https://github.com/vnghia/nghe/pull/644))
- add api key authentication ([#641](https://github.com/vnghia/nghe/pull/641))
- add support for unsync external lyrics ([#633](https://github.com/vnghia/nghe/pull/633))
- add external lyric full scan mode ([#630](https://github.com/vnghia/nghe/pull/630))
- WIP backend/lyrics: add support for lyrics ([#607](https://github.com/vnghia/nghe/pull/607))
- return main artist while querying album ([#601](https://github.com/vnghia/nghe/pull/601))
- return main artist in id3 song ([#600](https://github.com/vnghia/nghe/pull/600))
- return album/album id for track when querying album ([#599](https://github.com/vnghia/nghe/pull/599))
- add scan full information ([#596](https://github.com/vnghia/nghe/pull/596))
- add lastfm integration ([#595](https://github.com/vnghia/nghe/pull/595))
- fix rustfmt and clippy ([#584](https://github.com/vnghia/nghe/pull/584))
- add dir picture full mode ([#583](https://github.com/vnghia/nghe/pull/583))
- add tests for star related operations ([#582](https://github.com/vnghia/nghe/pull/582))
- add scan full file mode and multiple scans ([#576](https://github.com/vnghia/nghe/pull/576))
- ignore coverage for non-testable code ([#561](https://github.com/vnghia/nghe/pull/561))
- refactor auth from request ([#551](https://github.com/vnghia/nghe/pull/551))
- better tracing ([#546](https://github.com/vnghia/nghe/pull/546))
- fix flaky test in stream time offset ([#541](https://github.com/vnghia/nghe/pull/541))
- add update artist information ([#539](https://github.com/vnghia/nghe/pull/539))
- add json internal endpoint type ([#538](https://github.com/vnghia/nghe/pull/538))
- add get songs by genre ([#537](https://github.com/vnghia/nghe/pull/537))
- add Spotify integration for fetching cover art ([#536](https://github.com/vnghia/nghe/pull/536))
- better time serialize/deserialize ([#533](https://github.com/vnghia/nghe/pull/533))
- add api/handler for star related operations ([#526](https://github.com/vnghia/nghe/pull/526))
- add handler for playqueue related operations ([#524](https://github.com/vnghia/nghe/pull/524))
- add support for playlist related operations ([#523](https://github.com/vnghia/nghe/pull/523))
- use bincode serialize for better compatibility ([#520](https://github.com/vnghia/nghe/pull/520))
- remove old crates ([#515](https://github.com/vnghia/nghe/pull/515))
- add api/backend/proc-macro crate ([#412](https://github.com/vnghia/nghe/pull/412))
