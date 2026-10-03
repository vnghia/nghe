# Changelog

## [0.12.1](https://github.com/vnghia/nghe/compare/v0.14.1...v0.12.1) (2026-10-03)


### ⚠ BREAKING CHANGES

* rename nghe-backend to nghe ([#1125](https://github.com/vnghia/nghe/issues/1125))
* **api:** merge internal endpoint to normal endpoint ([#1121](https://github.com/vnghia/nghe/issues/1121))

### Features

* add immutable release ([#1138](https://github.com/vnghia/nghe/issues/1138)) ([70f29ca](https://github.com/vnghia/nghe/commit/70f29ca1a8c25b886decc0787022a0b342f27054))
* **api/backend:** add api for get music folder stats ([#796](https://github.com/vnghia/nghe/issues/796)) ([31c04a4](https://github.com/vnghia/nghe/commit/31c04a4257e7274326e755ff762d182f9c41d766))
* **api/backend:** add api for manage user ([#765](https://github.com/vnghia/nghe/issues/765)) ([2dfb0fc](https://github.com/vnghia/nghe/commit/2dfb0fc5517b287e73b6beb7083007c1c4597f0d))
* **api/backend:** add api for permission ([#772](https://github.com/vnghia/nghe/issues/772)) ([24840a6](https://github.com/vnghia/nghe/commit/24840a62f23862eb2ec2cb7e678a1e6375a3fd3c))
* **api:** merge internal endpoint to normal endpoint ([#1121](https://github.com/vnghia/nghe/issues/1121)) ([7a65aa4](https://github.com/vnghia/nghe/commit/7a65aa421fd789bd2447cb1f337a08ca1cad1f3d))
* **api:** remove binary encode for frontend ([#768](https://github.com/vnghia/nghe/issues/768)) ([0a373e3](https://github.com/vnghia/nghe/commit/0a373e3df9c46a3348cb09ddc57482184826741b))
* **backend:** add an endpoint for healthcheck ([#710](https://github.com/vnghia/nghe/issues/710)) ([3eb44e4](https://github.com/vnghia/nghe/commit/3eb44e41508ce09300e9443577f84512e1271afa))
* **backend:** add request decompresssion ([#770](https://github.com/vnghia/nghe/issues/770)) ([dba18e9](https://github.com/vnghia/nghe/commit/dba18e9c465fb80a77a9d7be789a4ded19b9c5a2))
* **backend:** add size support for get cover art ([#800](https://github.com/vnghia/nghe/issues/800)) ([517427c](https://github.com/vnghia/nghe/commit/517427c2dff6cbf9772f793278eb5e035bec4a68))
* **backend:** allow immutable for cache control ([#795](https://github.com/vnghia/nghe/issues/795)) ([afe0a30](https://github.com/vnghia/nghe/commit/afe0a30c6fb6a352226cb5f7497d889fb86b78c4))
* **backend:** embed frontend into the final binary ([#793](https://github.com/vnghia/nghe/issues/793)) ([67c2555](https://github.com/vnghia/nghe/commit/67c255588de6fcddb7142f673045d72e237b78bb))
* **backend:** simplify logging and add config ([#769](https://github.com/vnghia/nghe/issues/769)) ([3b1246c](https://github.com/vnghia/nghe/commit/3b1246c283e01b0f93ae51d381c05d3afc124f7a))
* **backend:** use atomic file write to write transcoding data ([#797](https://github.com/vnghia/nghe/issues/797)) ([7956fd0](https://github.com/vnghia/nghe/commit/7956fd01a29ad9ce730e0eab69ed1241745f0cdf))
* **ci:** automatically create new tag when merge release pr ([#1000](https://github.com/vnghia/nghe/issues/1000)) ([c1740e6](https://github.com/vnghia/nghe/commit/c1740e60c70c8319c718069a6a6b056f7f4fc6f4))
* **ci:** switch to nix for native dependencies ([#1064](https://github.com/vnghia/nghe/issues/1064)) ([d61fb78](https://github.com/vnghia/nghe/commit/d61fb787a21c41a58dbe830fa4db5cd21ecb3d3f))
* **ci:** use release-plz ([#1066](https://github.com/vnghia/nghe/issues/1066)) ([0d0638b](https://github.com/vnghia/nghe/commit/0d0638b1999c79d4cfbe52650a0f0bafe98ad1cf))
* **frontend:** add create/delete user ([#789](https://github.com/vnghia/nghe/issues/789)) ([5c8ddbd](https://github.com/vnghia/nghe/commit/5c8ddbd7140d06a633cc2c49fcc954c6151fc276))
* **frontend:** enable sri ([#801](https://github.com/vnghia/nghe/issues/801)) ([65f63e6](https://github.com/vnghia/nghe/commit/65f63e61dfca4a9fb584fc9376d1e86776fa1fee))
* **fs/s3:** replace aws s3 with more lightweight s3 ([#982](https://github.com/vnghia/nghe/issues/982)) ([fffa3d5](https://github.com/vnghia/nghe/commit/fffa3d5ab18cf25366122305993a703ead350a88))
* support build reproducible release binary ([#1135](https://github.com/vnghia/nghe/issues/1135)) ([4b7b7ba](https://github.com/vnghia/nghe/commit/4b7b7ba6813644f859b1488c5266093ba111c033))


### Bug Fixes

* **backend/extract:** remove redudant clone ([#1124](https://github.com/vnghia/nghe/issues/1124)) ([17ccf34](https://github.com/vnghia/nghe/commit/17ccf3459d4329dfaeb09b2413197129c0d79642))
* **backend:** increase cache duration for get cover art ([#756](https://github.com/vnghia/nghe/issues/756)) ([fb99f93](https://github.com/vnghia/nghe/commit/fb99f93df09d7db15b904a99bff446f4eaaf41bb))
* **backend:** propagate full to upsert artist picture ([#712](https://github.com/vnghia/nghe/issues/712)) ([e01b473](https://github.com/vnghia/nghe/commit/e01b473c8fda4baa956efeec0aee8ee121027ecd))
* **backend:** use fallback service instead of nest ([#791](https://github.com/vnghia/nghe/issues/791)) ([7eb9715](https://github.com/vnghia/nghe/commit/7eb9715614f2c833a3c6f74d6376d1b133d31e80))
* **ci:** add manual release pr ([#996](https://github.com/vnghia/nghe/issues/996)) ([24aab03](https://github.com/vnghia/nghe/commit/24aab03a9cf1be8e70a5c9dfd06885adcea4e511))
* **ci:** add missing release drafter config ([#999](https://github.com/vnghia/nghe/issues/999)) ([c21cfe5](https://github.com/vnghia/nghe/commit/c21cfe5060e84fd7d60d3133f7341c0eb160fd60))
* **ci:** add missing token ([#1001](https://github.com/vnghia/nghe/issues/1001)) ([4ed4664](https://github.com/vnghia/nghe/commit/4ed466429c7e6e36f5ad880ae6b19146e19a9ef5))
* **ci:** add permission to release pr ([#997](https://github.com/vnghia/nghe/issues/997)) ([750e17a](https://github.com/vnghia/nghe/commit/750e17a8e9b6dc2241a803b3acea40125b629895))
* **ci:** add prefix v to release drafter ([#1007](https://github.com/vnghia/nghe/issues/1007)) ([de3bf67](https://github.com/vnghia/nghe/commit/de3bf6704ae0b79aa25bb5ea2adaac67f09441e6))
* **ci:** add s regex option while matching commit message ([#1005](https://github.com/vnghia/nghe/issues/1005)) ([799e902](https://github.com/vnghia/nghe/commit/799e902f2f13b8455ffcbe43fb42652e9c93cc0b))
* **ci:** build with correct profile ([#1110](https://github.com/vnghia/nghe/issues/1110)) ([8c4ee2a](https://github.com/vnghia/nghe/commit/8c4ee2a91b2cd92efcb096335f94ab23d4954930))
* **ci:** correct package name ([09be13a](https://github.com/vnghia/nghe/commit/09be13ad7eb01cad1887620d2bd37ccb2ad0142f))
* **ci:** docker tag is wrong for manually triggered ([#1151](https://github.com/vnghia/nghe/issues/1151)) ([c35f17b](https://github.com/vnghia/nghe/commit/c35f17b81c9bd15e88339755f4c5a192a8cd05f7))
* **ci:** finalize release pipeline ([#1107](https://github.com/vnghia/nghe/issues/1107)) ([a3e4174](https://github.com/vnghia/nghe/commit/a3e41746652fd0c21a9f20c6a156365523e8507c))
* **ci:** fix ccbin path ([#1106](https://github.com/vnghia/nghe/issues/1106)) ([148010c](https://github.com/vnghia/nghe/commit/148010c2977a1a43e4c78e12bd2cab7126944717))
* **ci:** macos build artifact name ([#1112](https://github.com/vnghia/nghe/issues/1112)) ([7dcae03](https://github.com/vnghia/nghe/commit/7dcae03bf1b1cc2b90ae14ca7c084c96fa562e2f))
* **ci:** no manual rustc target ([#1069](https://github.com/vnghia/nghe/issues/1069)) ([690867b](https://github.com/vnghia/nghe/commit/690867bff62d0fecef86f3ce88b2649ac28e7daa))
* **ci:** only add v prefix in publish mode ([#1008](https://github.com/vnghia/nghe/issues/1008)) ([b0bc1f2](https://github.com/vnghia/nghe/commit/b0bc1f29fa049317b1f5b1ad4ae6b78c34b56991))
* **ci:** release pipeline is not triggered because of nested pipeline ([#1011](https://github.com/vnghia/nghe/issues/1011)) ([01d23d8](https://github.com/vnghia/nghe/commit/01d23d88987bd85ad153baef0b12eca6c099ca75))
* **ci:** release-plz group package version group ([#994](https://github.com/vnghia/nghe/issues/994)) ([f39dcc9](https://github.com/vnghia/nghe/commit/f39dcc9a4b57fea0886058aac660ebe96d4243ad))
* **ci:** release-plz only backend ([#995](https://github.com/vnghia/nghe/issues/995)) ([90e1e0c](https://github.com/vnghia/nghe/commit/90e1e0c44b4b17c8ea2f5da90d6ae8d9ee62a692))
* **ci:** replace minio with s3mock ([#947](https://github.com/vnghia/nghe/issues/947)) ([8370c9f](https://github.com/vnghia/nghe/commit/8370c9f9749ff3aafb5477fdef34a75ab2fe7895))
* **ci:** update release-please manifest and config ([fce79d2](https://github.com/vnghia/nghe/commit/fce79d20b5b3d5d02a72a52f0a99041f80fe5a88))
* **ci:** upload-release need git ([#1114](https://github.com/vnghia/nghe/issues/1114)) ([d5a9a31](https://github.com/vnghia/nghe/commit/d5a9a313485445d58441661954cefa3e940b64a1))
* **ci:** use nuget for vcpkg caching ([#1013](https://github.com/vnghia/nghe/issues/1013)) ([c4be99a](https://github.com/vnghia/nghe/commit/c4be99a1b166c30bdc9b1cff38a231561866ecc9))
* **ci:** use release please for release pipeline ([b790104](https://github.com/vnghia/nghe/commit/b790104a576d23a2d979d62ebf6c19cfd1845ddb))
* **ci:** use release-drafter feature ([#1003](https://github.com/vnghia/nghe/issues/1003)) ([e1cb4f0](https://github.com/vnghia/nghe/commit/e1cb4f09db499ed74f8929b63f79e7a10b9f06ff))
* **ci:** use release-please ([2cc299e](https://github.com/vnghia/nghe/commit/2cc299e7b9f7719ac90be4cf7dfabbc323988ee8))
* **ci:** use release-plz to release pipeline ([#993](https://github.com/vnghia/nghe/issues/993)) ([ca094b7](https://github.com/vnghia/nghe/commit/ca094b73178ebb30f300604131be3aaf54863fc6))
* **ci:** use repository owner instead of actor ([#1020](https://github.com/vnghia/nghe/issues/1020)) ([3012916](https://github.com/vnghia/nghe/commit/301291665d1b1c89f7ce99779485dadef30ab1bb))
* **ci:** use version group to release ([#1067](https://github.com/vnghia/nghe/issues/1067)) ([aeb9192](https://github.com/vnghia/nghe/commit/aeb919237393862bd0081ee7cfdeaa04a37e70ad))
* **ci:** windows and macos build ([#987](https://github.com/vnghia/nghe/issues/987)) ([c486375](https://github.com/vnghia/nghe/commit/c486375c29910ae41f004c408b17ff4a8b642603))
* **deps:** migrate to darling for proc-macro ([#1117](https://github.com/vnghia/nghe/issues/1117)) ([3ca42b8](https://github.com/vnghia/nghe/commit/3ca42b8841e7a70b8e1db4fbe0c51c5ddce18d94))
* **deps:** update aws-sdk-rust monorepo ([#617](https://github.com/vnghia/nghe/issues/617)) ([c70f817](https://github.com/vnghia/nghe/commit/c70f8175481a60b2f279731792b9cd8798a7815f))
* **deps:** update aws-sdk-rust monorepo ([#634](https://github.com/vnghia/nghe/issues/634)) ([e2e7137](https://github.com/vnghia/nghe/commit/e2e7137b91685f8e8a88b91ed62c4f69f27e296e))
* **deps:** update aws-sdk-rust monorepo ([#668](https://github.com/vnghia/nghe/issues/668)) ([487e9e0](https://github.com/vnghia/nghe/commit/487e9e0977002208a44b83e49f03d30615177911))
* **deps:** update aws-sdk-rust monorepo ([#690](https://github.com/vnghia/nghe/issues/690)) ([9befeb4](https://github.com/vnghia/nghe/commit/9befeb411034eaa3e7029e91254c90eca2294a91))
* **deps:** update aws-sdk-rust monorepo ([#729](https://github.com/vnghia/nghe/issues/729)) ([2235ae5](https://github.com/vnghia/nghe/commit/2235ae50689d62eeb562d4ba19bf2fbd20a7f28d))
* **deps:** update aws-sdk-rust monorepo ([#740](https://github.com/vnghia/nghe/issues/740)) ([3ec1de7](https://github.com/vnghia/nghe/commit/3ec1de7ca34571d59345f7efe1434f737f838787))
* **deps:** update aws-sdk-rust monorepo ([#758](https://github.com/vnghia/nghe/issues/758)) ([9d85e80](https://github.com/vnghia/nghe/commit/9d85e800eb06ca73edaed3c4f09d1899bb79631a))
* **deps:** update aws-sdk-rust monorepo ([#782](https://github.com/vnghia/nghe/issues/782)) ([b9fe028](https://github.com/vnghia/nghe/commit/b9fe0289f7234a962565a8984b431aaeaaf88b5f))
* **deps:** update aws-sdk-rust monorepo ([#818](https://github.com/vnghia/nghe/issues/818)) ([8fb15ef](https://github.com/vnghia/nghe/commit/8fb15efac853eb57472f15250af8aa5da3effd3d))
* **deps:** update axum monorepo ([#677](https://github.com/vnghia/nghe/issues/677)) ([37f7713](https://github.com/vnghia/nghe/commit/37f771360be4a1fa4b6421ffc92950f146bc3339))
* **deps:** update axum monorepo ([#965](https://github.com/vnghia/nghe/issues/965)) ([e3c5cce](https://github.com/vnghia/nghe/commit/e3c5ccedf6d486b7039ecb0b85550ff3c7ce03ce))
* **deps:** update dependency @tailwindcss/postcss to v4.0.0 ([#683](https://github.com/vnghia/nghe/issues/683)) ([750ae6a](https://github.com/vnghia/nghe/commit/750ae6a6c8ebfbb25bd937c1c5eedbe8de563ee5))
* **deps:** update dependency @tailwindcss/postcss to v4.0.0-beta.10 ([#680](https://github.com/vnghia/nghe/issues/680)) ([2f7c8a1](https://github.com/vnghia/nghe/commit/2f7c8a1d9beee0035b4c897d2660b83e1a317034))
* **deps:** update dependency @tailwindcss/postcss to v4.0.0-beta.9 ([#646](https://github.com/vnghia/nghe/issues/646)) ([0390d53](https://github.com/vnghia/nghe/commit/0390d533539c16981ae9d9cb4913dc7104255f31))
* **deps:** update dependency flowbite to v3.0.0 ([#689](https://github.com/vnghia/nghe/issues/689)) ([8fce133](https://github.com/vnghia/nghe/commit/8fce133858fd1293e9bbb74d3be100810024b598))
* **deps:** update dependency flowbite to v3.0.0-beta.3 ([#688](https://github.com/vnghia/nghe/issues/688)) ([ed8c38a](https://github.com/vnghia/nghe/commit/ed8c38ad83d63279b962fa6cad3274035dd4af95))
* **deps:** update dependency flowbite to v3.1.1 ([#708](https://github.com/vnghia/nghe/issues/708)) ([e2bb73a](https://github.com/vnghia/nghe/commit/e2bb73a7c737b034c6d52cf260f3b8d7d49137fd))
* **deps:** update dependency flowbite to v3.1.2 ([#717](https://github.com/vnghia/nghe/issues/717)) ([ee857f3](https://github.com/vnghia/nghe/commit/ee857f357893604bce2dbd8e99e7f27164a2f378))
* **deps:** update dependency tailwindcss to v4.0.0-beta.10 ([#681](https://github.com/vnghia/nghe/issues/681)) ([ef79536](https://github.com/vnghia/nghe/commit/ef795366286a430f8c9a8f72802917246ccac626))
* **deps:** update dependency tailwindcss to v4.0.0-beta.9 ([#648](https://github.com/vnghia/nghe/issues/648)) ([aa5ce50](https://github.com/vnghia/nghe/commit/aa5ce5014d3acfe95ff570b7b489864264139e33))
* **deps:** update diesel_derives digest to 7629365 ([#891](https://github.com/vnghia/nghe/issues/891)) ([228ffd4](https://github.com/vnghia/nghe/commit/228ffd468912a3c507b12bad646c821f8e907a31))
* **deps:** update rust crate async-walkdir to v2.1.0 ([#695](https://github.com/vnghia/nghe/issues/695)) ([8bf587a](https://github.com/vnghia/nghe/commit/8bf587aa4c6beb1302dfdf736d844987cedf2c3a))
* **deps:** update rust crate atomic-write-file to 0.3.0 ([#972](https://github.com/vnghia/nghe/issues/972)) ([f5da00c](https://github.com/vnghia/nghe/commit/f5da00ce0996b26d65d111004782f6c832743ee2))
* **deps:** update rust crate aws-sdk-s3 to v1.70.0 ([#685](https://github.com/vnghia/nghe/issues/685)) ([1e9b29c](https://github.com/vnghia/nghe/commit/1e9b29cd40d1957dfcfb2ec571a50e8de6ba38c5))
* **deps:** update rust crate aws-sdk-s3 to v1.72.0 ([#699](https://github.com/vnghia/nghe/issues/699)) ([1b8a5d6](https://github.com/vnghia/nghe/commit/1b8a5d6cef061ed0b74569082bf949098c5fa0ee))
* **deps:** update rust crate aws-sdk-s3 to v1.81.0 ([#821](https://github.com/vnghia/nghe/issues/821)) ([2e21527](https://github.com/vnghia/nghe/commit/2e21527f2b96d63cefb252778c6d181e538db946))
* **deps:** update rust crate aws-sdk-s3 to v1.82.0 ([#822](https://github.com/vnghia/nghe/issues/822)) ([c74eea5](https://github.com/vnghia/nghe/commit/c74eea5d90dbfc6685d892c7c6a16b6a22ef8861))
* **deps:** update rust crate aws-smithy-runtime to v1.7.6 ([#612](https://github.com/vnghia/nghe/issues/612)) ([5064c63](https://github.com/vnghia/nghe/commit/5064c63f5dce7fa7cdfc97c5bcb121d05c95dae4))
* **deps:** update rust crate aws-smithy-runtime to v1.7.7 ([#662](https://github.com/vnghia/nghe/issues/662)) ([397ffed](https://github.com/vnghia/nghe/commit/397ffed094b623788d8e605d1f2c6bd2f73e589d))
* **deps:** update rust crate aws-smithy-runtime to v1.7.7 ([#665](https://github.com/vnghia/nghe/issues/665)) ([f228330](https://github.com/vnghia/nghe/commit/f228330bbea185abf3a22c99e4cb7c40175ee088))
* **deps:** update rust crate aws-smithy-runtime to v1.8.0 ([#776](https://github.com/vnghia/nghe/issues/776)) ([d77f538](https://github.com/vnghia/nghe/commit/d77f538e09c89adebfa1f28d8eb9ec7377cee187))
* **deps:** update rust crate aws-smithy-runtime to v1.8.3 ([#842](https://github.com/vnghia/nghe/issues/842)) ([cd02d9b](https://github.com/vnghia/nghe/commit/cd02d9b434729e0bb695c95ffd363f73519d3b21))
* **deps:** update rust crate axum to v0.8.1 ([#626](https://github.com/vnghia/nghe/issues/626)) ([a829071](https://github.com/vnghia/nghe/commit/a8290719cdf757f1aeee04654dff23083051faaf))
* **deps:** update rust crate axum to v0.8.3 ([#820](https://github.com/vnghia/nghe/issues/820)) ([8bb88d4](https://github.com/vnghia/nghe/commit/8bb88d4e06a66b1db8a6fae01262708f477d78a3))
* **deps:** update rust crate axum to v0.8.4 ([#849](https://github.com/vnghia/nghe/issues/849)) ([361738b](https://github.com/vnghia/nghe/commit/361738bfbcaf257bea5947d0504a10aafcd5c96e))
* **deps:** update rust crate axum-extra to v0.10.0 ([#623](https://github.com/vnghia/nghe/issues/623)) ([5efdac8](https://github.com/vnghia/nghe/commit/5efdac82275ef44d901880283675fa29d2707cb7))
* **deps:** update rust crate codee to 0.3.0 ([#739](https://github.com/vnghia/nghe/issues/739)) ([503b559](https://github.com/vnghia/nghe/commit/503b559ed3c3f3bf347b7145ec98fcb2ae6133a5))
* **deps:** update rust crate codee to v0.3.2 ([#905](https://github.com/vnghia/nghe/issues/905)) ([57d38a4](https://github.com/vnghia/nghe/commit/57d38a437ae879a553f7833734a478c2ee89b6ab))
* **deps:** update rust crate convert_case to 0.7.0 ([#654](https://github.com/vnghia/nghe/issues/654)) ([66b6668](https://github.com/vnghia/nghe/commit/66b6668cfa539a31e71b8438f028d120ed65527b))
* **deps:** update rust crate convert_case to 0.8.0 ([#744](https://github.com/vnghia/nghe/issues/744)) ([ec05f63](https://github.com/vnghia/nghe/commit/ec05f6331639d2a3c41da1963eaf4882a4d4680e))
* **deps:** update rust crate convert_case to v0.7.1 ([#659](https://github.com/vnghia/nghe/issues/659)) ([3ee3067](https://github.com/vnghia/nghe/commit/3ee30671fff0cd7cc086fb03e0a992211ef6face))
* **deps:** update rust crate diesel to v2.2.10 ([#844](https://github.com/vnghia/nghe/issues/844)) ([e659a01](https://github.com/vnghia/nghe/commit/e659a018ca5f1fa11a15d6488d43aeb3a0496587))
* **deps:** update rust crate diesel to v2.2.11 ([#902](https://github.com/vnghia/nghe/issues/902)) ([6e0eb3c](https://github.com/vnghia/nghe/commit/6e0eb3c92b78a0ca7676c9998b0399f54cd2c215))
* **deps:** update rust crate diesel to v2.2.12 ([#916](https://github.com/vnghia/nghe/issues/916)) ([bff374e](https://github.com/vnghia/nghe/commit/bff374ee08da5f6a54ecb6bafec3b3e85fa467ae))
* **deps:** update rust crate diesel to v2.2.9 ([#826](https://github.com/vnghia/nghe/issues/826)) ([c6299ce](https://github.com/vnghia/nghe/commit/c6299ce501d5cdda7999eab33284a292c12e8678))
* **deps:** update rust crate diesel-async to 0.6.0 ([#912](https://github.com/vnghia/nghe/issues/912)) ([0798bf3](https://github.com/vnghia/nghe/commit/0798bf3755202b4d387b742df0d414722309a49a))
* **deps:** update rust crate diesel-async to 0.8.0 ([#1056](https://github.com/vnghia/nghe/issues/1056)) ([089854f](https://github.com/vnghia/nghe/commit/089854fba1bf1b66721fae6a70601b0eac7811df))
* **deps:** update rust crate diesel-async to 0.9.0 ([#1074](https://github.com/vnghia/nghe/issues/1074)) ([98005b7](https://github.com/vnghia/nghe/commit/98005b784067312b59705ef7dbe7ac58d84e2211))
* **deps:** update rust crate futures-lite to v2.6.0 ([#655](https://github.com/vnghia/nghe/issues/655)) ([1010bcc](https://github.com/vnghia/nghe/commit/1010bccb6066bcdeb3f435ea8c7d6e552c3102f9))
* **deps:** update rust crate futures-lite to v2.6.1 ([#929](https://github.com/vnghia/nghe/issues/929)) ([7818e2a](https://github.com/vnghia/nghe/commit/7818e2a350d80c4894b542c13183ed49dfbbfc1a))
* **deps:** update rust crate image to v0.25.6 ([#816](https://github.com/vnghia/nghe/issues/816)) ([06f6258](https://github.com/vnghia/nghe/commit/06f6258e47a68ff676d8aacea3fff4cb5a0687fa))
* **deps:** update rust crate indexmap to v2.10.0 ([#908](https://github.com/vnghia/nghe/issues/908)) ([3944f4c](https://github.com/vnghia/nghe/commit/3944f4c1292faaa1c81f64e352f6a1ba7a93a4e2))
* **deps:** update rust crate indexmap to v2.7.1 ([#674](https://github.com/vnghia/nghe/issues/674)) ([a64fe48](https://github.com/vnghia/nghe/commit/a64fe48083be011bd6ddfddaa26e4f7afdd4a3ba))
* **deps:** update rust crate indexmap to v2.8.0 ([#777](https://github.com/vnghia/nghe/issues/777)) ([2c63662](https://github.com/vnghia/nghe/commit/2c63662b40ee3c14c2dfc2eb8fdcd63e48bbb70b))
* **deps:** update rust crate indexmap to v2.9.0 ([#828](https://github.com/vnghia/nghe/issues/828)) ([5aa4242](https://github.com/vnghia/nghe/commit/5aa42422216e90caac3a9dd43593be9cddeb62d2))
* **deps:** update rust crate leptos to 0.8.0 ([#853](https://github.com/vnghia/nghe/issues/853)) ([427af30](https://github.com/vnghia/nghe/commit/427af302cc111a8984e9cbbb8338c1cf61d4dcfe))
* **deps:** update rust crate leptos to v0.7.3 ([#635](https://github.com/vnghia/nghe/issues/635)) ([4357e12](https://github.com/vnghia/nghe/commit/4357e12df18942c208837906a4d4dee8e74de196))
* **deps:** update rust crate leptos to v0.7.4 ([#669](https://github.com/vnghia/nghe/issues/669)) ([b9df9cd](https://github.com/vnghia/nghe/commit/b9df9cdc32766ed7d2f2d559522a7c8aa6e187ab))
* **deps:** update rust crate leptos to v0.7.7 ([#730](https://github.com/vnghia/nghe/issues/730)) ([db04f39](https://github.com/vnghia/nghe/commit/db04f3997253111611409c6c3a852c2d0c370cb0))
* **deps:** update rust crate leptos_router to 0.8.0 ([#854](https://github.com/vnghia/nghe/issues/854)) ([dea18b5](https://github.com/vnghia/nghe/commit/dea18b5381a63d24e6504a7ade030240a1dae7af))
* **deps:** update rust crate leptos_router to v0.7.4 ([#671](https://github.com/vnghia/nghe/issues/671)) ([ef647d3](https://github.com/vnghia/nghe/commit/ef647d33a253a53186c32bb00e692014d8b4ad1a))
* **deps:** update rust crate leptos_router to v0.7.7 ([#732](https://github.com/vnghia/nghe/issues/732)) ([f1bbf08](https://github.com/vnghia/nghe/commit/f1bbf086d90ee56dfd002b107572eb50168cedb1))
* **deps:** update rust crate leptos_router to v0.7.8 ([#810](https://github.com/vnghia/nghe/issues/810)) ([679c015](https://github.com/vnghia/nghe/commit/679c0154ca5310fc3a65118f0e7eb1abbc446e21))
* **deps:** update rust crate leptos-use to 0.16.0 ([#897](https://github.com/vnghia/nghe/issues/897)) ([0c5fd62](https://github.com/vnghia/nghe/commit/0c5fd629b6c26ef3e6f68f0e7ebafcfc91d0b55a))
* **deps:** update rust crate leptos-use to v0.15.4 ([#667](https://github.com/vnghia/nghe/issues/667)) ([1dab1ca](https://github.com/vnghia/nghe/commit/1dab1ca1d488584a03796d814bdc94a996a45344))
* **deps:** update rust crate leptos-use to v0.15.5 ([#670](https://github.com/vnghia/nghe/issues/670)) ([9d5f99d](https://github.com/vnghia/nghe/commit/9d5f99d8fcdd2e192a50a387f04e00b49fc023b8))
* **deps:** update rust crate leptos-use to v0.15.6 ([#731](https://github.com/vnghia/nghe/issues/731)) ([6e2a9ff](https://github.com/vnghia/nghe/commit/6e2a9fff19fb9b189309cae5dcf58f51d1f5040c))
* **deps:** update rust crate leptos-use to v0.15.7 ([#803](https://github.com/vnghia/nghe/issues/803)) ([ab00a21](https://github.com/vnghia/nghe/commit/ab00a2139efa217b62406f601d9866458924bbaa))
* **deps:** update rust crate leptos-use to v0.16.1 ([#901](https://github.com/vnghia/nghe/issues/901)) ([c238c87](https://github.com/vnghia/nghe/commit/c238c8701b9709af838555348a05544f4bac26f9))
* **deps:** update rust crate leptos-use to v0.16.2 ([#903](https://github.com/vnghia/nghe/issues/903)) ([3058c11](https://github.com/vnghia/nghe/commit/3058c118a31c1f15b87d7deaa71825af15b7550e))
* **deps:** update rust crate lofty to 0.22.0 ([#637](https://github.com/vnghia/nghe/issues/637)) ([6377718](https://github.com/vnghia/nghe/commit/63777188bf6e7004aac4a461ec51482a7e3194c7))
* **deps:** update rust crate lofty to 0.23.0 ([#986](https://github.com/vnghia/nghe/issues/986)) ([a98d90e](https://github.com/vnghia/nghe/commit/a98d90e5df51c044f042caab2ba956d1ce84c5c5))
* **deps:** update rust crate lofty to 0.25.0 ([#1063](https://github.com/vnghia/nghe/issues/1063)) ([d9e6ae7](https://github.com/vnghia/nghe/commit/d9e6ae7a17727ef23c1d3933904c8d62deb010d5))
* **deps:** update rust crate lofty to v0.22.1 ([#653](https://github.com/vnghia/nghe/issues/653)) ([7eadc16](https://github.com/vnghia/nghe/commit/7eadc162e099df172baf88003401743921f9fed5))
* **deps:** update rust crate lofty to v0.22.2 ([#733](https://github.com/vnghia/nghe/issues/733)) ([db6e202](https://github.com/vnghia/nghe/commit/db6e202a171dbac7a529a019a2c15543bd4dc8c1))
* **deps:** update rust crate lofty to v0.22.3 ([#827](https://github.com/vnghia/nghe/issues/827)) ([90e918b](https://github.com/vnghia/nghe/commit/90e918bbff7657ca2bad4f21de0b133a4eb630d3))
* **deps:** update rust crate lofty to v0.22.4 ([#848](https://github.com/vnghia/nghe/issues/848)) ([fb7161a](https://github.com/vnghia/nghe/commit/fb7161a7f6389ecb3f75d8b8ece01584234bc172))
* **deps:** update rust crate loole to v0.4.1 ([#847](https://github.com/vnghia/nghe/issues/847)) ([21ab28b](https://github.com/vnghia/nghe/commit/21ab28b5a6660c9ead51f0931a24955e26014828))
* **deps:** update rust crate md5 to 0.8.0 ([#907](https://github.com/vnghia/nghe/issues/907)) ([9e3cfd5](https://github.com/vnghia/nghe/commit/9e3cfd5b6b3e53417b6c3abc169264452f34e4be))
* **deps:** update rust crate memory-serve to v1.1.0 ([#813](https://github.com/vnghia/nghe/issues/813)) ([b03f16a](https://github.com/vnghia/nghe/commit/b03f16aed8d602a9b7876d3276e8a89d02f8ee04))
* **deps:** update rust crate memory-serve to v1.2.1 ([#815](https://github.com/vnghia/nghe/issues/815)) ([cb80720](https://github.com/vnghia/nghe/commit/cb80720e2002f292311bfb457f3098c6ad04d621))
* **deps:** update rust crate mimalloc to v0.1.44 ([#806](https://github.com/vnghia/nghe/issues/806)) ([dd5e671](https://github.com/vnghia/nghe/commit/dd5e67125e632564472125134a2579371e12e65b))
* **deps:** update rust crate mimalloc to v0.1.46 ([#824](https://github.com/vnghia/nghe/issues/824)) ([f2b6622](https://github.com/vnghia/nghe/commit/f2b6622d0d1b33b141580895ca74193f0b5db6b4))
* **deps:** update rust crate mimalloc to v0.1.47 ([#900](https://github.com/vnghia/nghe/issues/900)) ([edcff75](https://github.com/vnghia/nghe/commit/edcff75814a86156f761156da3852bcb301a5aed))
* **deps:** update rust crate o2o to v0.5.1 ([#614](https://github.com/vnghia/nghe/issues/614)) ([b19dd43](https://github.com/vnghia/nghe/commit/b19dd4387ba53e64bb857b7ec5895c0211ea794c))
* **deps:** update rust crate o2o to v0.5.2 ([#652](https://github.com/vnghia/nghe/issues/652)) ([4ecdb5f](https://github.com/vnghia/nghe/commit/4ecdb5f48c7551e4b57c8a091a6c86787e7965e8))
* **deps:** update rust crate o2o to v0.5.3 ([#678](https://github.com/vnghia/nghe/issues/678)) ([16f978d](https://github.com/vnghia/nghe/commit/16f978d733a308db3da3f0f2103c005df1d67940))
* **deps:** update rust crate o2o to v0.5.4 ([#851](https://github.com/vnghia/nghe/issues/851)) ([b92401d](https://github.com/vnghia/nghe/commit/b92401d173174774020e67f77fa74b64356f5710))
* **deps:** update rust crate proc-macro2 to v1.0.93 ([#651](https://github.com/vnghia/nghe/issues/651)) ([377df35](https://github.com/vnghia/nghe/commit/377df352b85d89e8f0c61e4096242fbdfd0fe86f))
* **deps:** update rust crate proc-macro2 to v1.0.94 ([#751](https://github.com/vnghia/nghe/issues/751)) ([7678b06](https://github.com/vnghia/nghe/commit/7678b06f3cdcac1494cfb17e76d3834611f76577))
* **deps:** update rust crate proc-macro2 to v1.0.95 ([#836](https://github.com/vnghia/nghe/issues/836)) ([8237f81](https://github.com/vnghia/nghe/commit/8237f81d76e1c08c944eb8d4329d015ee54008ae))
* **deps:** update rust crate quote to v1.0.38 ([#610](https://github.com/vnghia/nghe/issues/610)) ([795780b](https://github.com/vnghia/nghe/commit/795780b30e4c7c4e1195ff0f4f90fdffa2969eb2))
* **deps:** update rust crate quote to v1.0.39 ([#752](https://github.com/vnghia/nghe/issues/752)) ([27e0234](https://github.com/vnghia/nghe/commit/27e023489b69007b6de5296a9488e2fb5b2a0e6e))
* **deps:** update rust crate quote to v1.0.40 ([#781](https://github.com/vnghia/nghe/issues/781)) ([a15a04a](https://github.com/vnghia/nghe/commit/a15a04a053cc6ca4ac58745cad8a84e4c51ba033))
* **deps:** update rust crate rsmpeg to 0.18.0 ([#893](https://github.com/vnghia/nghe/issues/893)) ([c51da0d](https://github.com/vnghia/nghe/commit/c51da0d8a82484caa025b63d8aecf8dd174f18fb))
* **deps:** update rust crate rsmpeg to v0.15.2 ([#861](https://github.com/vnghia/nghe/issues/861)) ([6cf866c](https://github.com/vnghia/nghe/commit/6cf866cc2be27531bd04ef8b3412360ee892224a))
* **deps:** update rust crate rsmpeg to v0.15.2 ([#870](https://github.com/vnghia/nghe/issues/870)) ([f6ef699](https://github.com/vnghia/nghe/commit/f6ef69975eadc8f0e0dc47f2c45ba9d48cc13e28))
* **deps:** update rust crate rsmpeg to v0.15.2 ([#872](https://github.com/vnghia/nghe/issues/872)) ([89212be](https://github.com/vnghia/nghe/commit/89212beaa8c3129ab50eaa06fd69153a7a162983))
* **deps:** update rust crate rsmpeg to v0.15.2 ([#874](https://github.com/vnghia/nghe/issues/874)) ([7d69843](https://github.com/vnghia/nghe/commit/7d69843d56ad67a91fc21a0327b8bcea9147fb62))
* **deps:** update rust crate rspotify to 0.14.0 ([#622](https://github.com/vnghia/nghe/issues/622)) ([310e6d8](https://github.com/vnghia/nghe/commit/310e6d897944e0b5dde4f147cd0021b951529e81))
* **deps:** update rust crate rspotify to 0.15.0 ([#914](https://github.com/vnghia/nghe/issues/914)) ([396ab9f](https://github.com/vnghia/nghe/commit/396ab9ff99add77ed3b2fe9285db7d79efcc169a))
* **deps:** update rust crate rspotify to 0.16.0 ([#1057](https://github.com/vnghia/nghe/issues/1057)) ([cea6007](https://github.com/vnghia/nghe/commit/cea600787522d13c57f8f773451a181298cbcce0))
* **deps:** update rust crate rspotify to v0.15.1 ([#932](https://github.com/vnghia/nghe/issues/932)) ([35c79b6](https://github.com/vnghia/nghe/commit/35c79b6b896ff2b588221b78339ef505f6b2cb97))
* **deps:** update rust crate syn to v2.0.100 ([#775](https://github.com/vnghia/nghe/issues/775)) ([c74ba14](https://github.com/vnghia/nghe/commit/c74ba147173a2b5d7665c6a5f9d45e088857efce))
* **deps:** update rust crate syn to v2.0.101 ([#846](https://github.com/vnghia/nghe/issues/846)) ([53faf53](https://github.com/vnghia/nghe/commit/53faf5337064b91096653bfaeb6c90a98a08bf07))
* **deps:** update rust crate syn to v2.0.102 ([#894](https://github.com/vnghia/nghe/issues/894)) ([90df294](https://github.com/vnghia/nghe/commit/90df29424688f479ef470440350fdd274514a45e))
* **deps:** update rust crate syn to v2.0.103 ([#898](https://github.com/vnghia/nghe/issues/898)) ([3b7f25f](https://github.com/vnghia/nghe/commit/3b7f25fedde57de36534390b1a3ae167b2794d82))
* **deps:** update rust crate syn to v2.0.104 ([#904](https://github.com/vnghia/nghe/issues/904)) ([702830a](https://github.com/vnghia/nghe/commit/702830a9670cf9844822f134aa7321732b766be4))
* **deps:** update rust crate syn to v2.0.106 ([#933](https://github.com/vnghia/nghe/issues/933)) ([839b4f4](https://github.com/vnghia/nghe/commit/839b4f44e54b5ae627de53c9820c2f4cbb4ff0b4))
* **deps:** update rust crate syn to v2.0.92 ([#615](https://github.com/vnghia/nghe/issues/615)) ([fc53fe6](https://github.com/vnghia/nghe/commit/fc53fe6aa5c8e25767961fd62fd29844d9878065))
* **deps:** update rust crate syn to v2.0.93 ([#619](https://github.com/vnghia/nghe/issues/619)) ([ec66caf](https://github.com/vnghia/nghe/commit/ec66cafd9aedd820a1428db8dd8be392ffd9d243))
* **deps:** update rust crate syn to v2.0.94 ([#628](https://github.com/vnghia/nghe/issues/628)) ([49ed365](https://github.com/vnghia/nghe/commit/49ed365c607cc43bc7ae98d9008ec5a6ad500447))
* **deps:** update rust crate syn to v2.0.95 ([#636](https://github.com/vnghia/nghe/issues/636)) ([07d78c9](https://github.com/vnghia/nghe/commit/07d78c9c17f35ee983be09804eceb32527648e1a))
* **deps:** update rust crate syn to v2.0.96 ([#643](https://github.com/vnghia/nghe/issues/643)) ([b97393d](https://github.com/vnghia/nghe/commit/b97393d6ad571bd079b6ab731d697e17ce72a453))
* **deps:** update rust crate syn to v2.0.98 ([#734](https://github.com/vnghia/nghe/issues/734)) ([977bcff](https://github.com/vnghia/nghe/commit/977bcffb37e38ae4841c94733b2274812596f63e))
* **deps:** update rust crate syn to v2.0.99 ([#753](https://github.com/vnghia/nghe/issues/753)) ([eac9e9f](https://github.com/vnghia/nghe/commit/eac9e9f8d608b6ea9594c93741d855cdfb9f9836))
* **deps:** update rust crate tokio to v1.43.0 ([#639](https://github.com/vnghia/nghe/issues/639)) ([d68b9d7](https://github.com/vnghia/nghe/commit/d68b9d7fd26a9260308c076d06325918636cbd7b))
* **deps:** update rust crate tokio to v1.44.0 ([#773](https://github.com/vnghia/nghe/issues/773)) ([f43f41f](https://github.com/vnghia/nghe/commit/f43f41fe2a5832278f7f6fe46d79c960817fc95c))
* **deps:** update rust crate tokio to v1.44.1 ([#786](https://github.com/vnghia/nghe/issues/786)) ([1e7a4c7](https://github.com/vnghia/nghe/commit/1e7a4c718213c20fbdfa63bcb5c85b985609a78b))
* **deps:** update rust crate tokio to v1.44.2 ([#829](https://github.com/vnghia/nghe/issues/829)) ([1edeafa](https://github.com/vnghia/nghe/commit/1edeafaafee07a96edc1501969628a9eff8c12cf))
* **deps:** update rust crate tokio to v1.45.0 ([#856](https://github.com/vnghia/nghe/issues/856)) ([7df1650](https://github.com/vnghia/nghe/commit/7df1650718e6204874612275a1b250748bf3aa61))
* **deps:** update rust crate tokio to v1.45.1 ([#862](https://github.com/vnghia/nghe/issues/862)) ([59c83fb](https://github.com/vnghia/nghe/commit/59c83fb0b4c63b366016706cc3da8da23426936c))
* **deps:** update rust crate tokio to v1.45.1 ([#875](https://github.com/vnghia/nghe/issues/875)) ([57de6bc](https://github.com/vnghia/nghe/commit/57de6bc1993f8efa12b0545ef56d94a37b7b8648))
* **deps:** update rust crate tokio to v1.45.1 ([#876](https://github.com/vnghia/nghe/issues/876)) ([67e4082](https://github.com/vnghia/nghe/commit/67e4082937daf31949f5446c65a35bb10788da1e))
* **deps:** update rust crate tokio to v1.45.1 ([#877](https://github.com/vnghia/nghe/issues/877)) ([f28a13b](https://github.com/vnghia/nghe/commit/f28a13b831742e01e1e6904e253108c187a6184a))
* **deps:** update rust crate tokio to v1.45.1 ([#881](https://github.com/vnghia/nghe/issues/881)) ([8ec74c4](https://github.com/vnghia/nghe/commit/8ec74c4bce2f1b14ca6b444c1f6fb15a4bbcb381))
* **deps:** update rust crate tokio to v1.45.1 ([#883](https://github.com/vnghia/nghe/issues/883)) ([40424ad](https://github.com/vnghia/nghe/commit/40424ad1a9c13e1a01b0402538479f41ef5590b8))
* **deps:** update rust crate tokio to v1.46.0 ([#911](https://github.com/vnghia/nghe/issues/911)) ([2cf9563](https://github.com/vnghia/nghe/commit/2cf9563e00ec9ce72bd1eda3741491157c202b33))
* **deps:** update rust crate tokio to v1.47.1 ([#913](https://github.com/vnghia/nghe/issues/913)) ([1bebe4e](https://github.com/vnghia/nghe/commit/1bebe4e571ea8953f99191fb2a666a728d96e796))
* **deps:** update rust crate tokio-util to v0.7.14 ([#787](https://github.com/vnghia/nghe/issues/787)) ([4d90278](https://github.com/vnghia/nghe/commit/4d902780f609b770e5ed2b0ac9b21fc39a3a9366))
* **deps:** update rust crate tokio-util to v0.7.15 ([#841](https://github.com/vnghia/nghe/issues/841)) ([44b2475](https://github.com/vnghia/nghe/commit/44b24758ba1bdcf7d456cf32dc408da248c351bd))
* **deps:** update rust crate tokio-util to v0.7.16 ([#934](https://github.com/vnghia/nghe/issues/934)) ([ad77a49](https://github.com/vnghia/nghe/commit/ad77a494a65e30996db71db283cc30c88c9429ba))
* **deps:** update rust crate tower-http to 0.7.0 ([#1075](https://github.com/vnghia/nghe/issues/1075)) ([3a599f5](https://github.com/vnghia/nghe/commit/3a599f55cac4fd7460b467a8a4d85d6ffc698fbe))
* **deps:** update rust crate tower-http to v0.6.4 ([#857](https://github.com/vnghia/nghe/issues/857)) ([c45d231](https://github.com/vnghia/nghe/commit/c45d2319d6682bf1523bff742fe75832f45c9dbe))
* **deps:** update rust crate tower-http to v0.6.6 ([#880](https://github.com/vnghia/nghe/issues/880)) ([ddef4e3](https://github.com/vnghia/nghe/commit/ddef4e31f71491600260b66f4c16c7af47c34788))
* **deps:** update rust crate tracing-subscriber to v0.3.20 ([#935](https://github.com/vnghia/nghe/issues/935)) ([6782055](https://github.com/vnghia/nghe/commit/67820557a15fbea99a3cd4ff3c1577590e3f2021))
* **deps:** update rust crate typed-path to 0.11.0 ([#845](https://github.com/vnghia/nghe/issues/845)) ([e7b609d](https://github.com/vnghia/nghe/commit/e7b609d975a2d392cad8edb701b3d8d1e67cad0e))
* **deps:** update rust crate typed-path to 0.12.0 ([#966](https://github.com/vnghia/nghe/issues/966)) ([5cf65c7](https://github.com/vnghia/nghe/commit/5cf65c7c2a5f2a898b7088bda2e79ff27ee81752))
* **deps:** update rust crate xxhash-rust to v0.8.14 ([#608](https://github.com/vnghia/nghe/issues/608)) ([09227f1](https://github.com/vnghia/nghe/commit/09227f11cce2021fab1e1b53ded47c362d191db3))
* **deps:** update rust crate xxhash-rust to v0.8.15 ([#620](https://github.com/vnghia/nghe/issues/620)) ([b2ad749](https://github.com/vnghia/nghe/commit/b2ad749bc077df3d6902e2537b80ac829fd624e1))
* **deps:** update tailwindcss monorepo to v4.0.1 ([#698](https://github.com/vnghia/nghe/issues/698)) ([65db5d9](https://github.com/vnghia/nghe/commit/65db5d9cf65f91a388f8e1b7d0d733c52d91624a))
* **deps:** update tailwindcss monorepo to v4.0.1 ([#701](https://github.com/vnghia/nghe/issues/701)) ([26d46de](https://github.com/vnghia/nghe/commit/26d46de19e3b8fad74e4e99b2662a4f09e4483bc))
* **deps:** update tailwindcss monorepo to v4.0.10 ([#755](https://github.com/vnghia/nghe/issues/755)) ([ebc1dd0](https://github.com/vnghia/nghe/commit/ebc1dd03594e1e780defb3ef7f32ec6bd3d09c79))
* **deps:** update tailwindcss monorepo to v4.0.11 ([#766](https://github.com/vnghia/nghe/issues/766)) ([b08782f](https://github.com/vnghia/nghe/commit/b08782fa8d0ab110c81576b6019421b08d384dd8))
* **deps:** update tailwindcss monorepo to v4.0.12 ([#771](https://github.com/vnghia/nghe/issues/771)) ([5be8ade](https://github.com/vnghia/nghe/commit/5be8adee527eed4961a942ca734f4cb4b053384f))
* **deps:** update tailwindcss monorepo to v4.0.13 ([#779](https://github.com/vnghia/nghe/issues/779)) ([f6fc259](https://github.com/vnghia/nghe/commit/f6fc25959199171dc97cbe66eee8c9d97f870939))
* **deps:** update tailwindcss monorepo to v4.0.14 ([#788](https://github.com/vnghia/nghe/issues/788)) ([df42302](https://github.com/vnghia/nghe/commit/df42302df341b6bcd349eca80e364fc5ea435849))
* **deps:** update tailwindcss monorepo to v4.0.15 ([#811](https://github.com/vnghia/nghe/issues/811)) ([59ee28f](https://github.com/vnghia/nghe/commit/59ee28fb4f4ff66548c969dc09ce4b9db76ec416))
* **deps:** update tailwindcss monorepo to v4.0.16 ([#817](https://github.com/vnghia/nghe/issues/817)) ([5ef9740](https://github.com/vnghia/nghe/commit/5ef9740fd65b530707a72c72c6269825162d5781))
* **deps:** update tailwindcss monorepo to v4.0.17 ([#819](https://github.com/vnghia/nghe/issues/819)) ([c4da25c](https://github.com/vnghia/nghe/commit/c4da25cf34c6049494e07f61a61d4f31e6a6005c))
* **deps:** update tailwindcss monorepo to v4.0.2 ([#706](https://github.com/vnghia/nghe/issues/706)) ([7084ff1](https://github.com/vnghia/nghe/commit/7084ff15d0a346613f5aa7f6e3d5d94d30c5eda3))
* **deps:** update tailwindcss monorepo to v4.0.3 ([#713](https://github.com/vnghia/nghe/issues/713)) ([1dc6632](https://github.com/vnghia/nghe/commit/1dc6632d9b049de45e7eb30d3a8122a25bf4fd26))
* **deps:** update tailwindcss monorepo to v4.0.4 ([#716](https://github.com/vnghia/nghe/issues/716)) ([0e7848d](https://github.com/vnghia/nghe/commit/0e7848de8068f2a99c129ea2fffd3375a2611fef))
* **deps:** update tailwindcss monorepo to v4.0.5 ([#718](https://github.com/vnghia/nghe/issues/718)) ([d448ced](https://github.com/vnghia/nghe/commit/d448ceda3356525990660fb36e17163e3810f195))
* **deps:** update tailwindcss monorepo to v4.0.6 ([#720](https://github.com/vnghia/nghe/issues/720)) ([2a94bbf](https://github.com/vnghia/nghe/commit/2a94bbfa82ce1d15d51e3c9620e06e8e86fec036))
* **deps:** update tailwindcss monorepo to v4.0.8 ([#723](https://github.com/vnghia/nghe/issues/723)) ([cbdc2e2](https://github.com/vnghia/nghe/commit/cbdc2e25dca2e95d764455a3fd9f5c9aa2c7683d))
* **deps:** update tailwindcss monorepo to v4.0.9 ([#742](https://github.com/vnghia/nghe/issues/742)) ([4339288](https://github.com/vnghia/nghe/commit/4339288bde2c27a237d9aa39bce7a0a4921452a5))
* **deps:** update tailwindcss monorepo to v4.1.10 ([#896](https://github.com/vnghia/nghe/issues/896)) ([07c94aa](https://github.com/vnghia/nghe/commit/07c94aa4710b64bbf85f705e02df4ba1406fa95d))
* **deps:** update tailwindcss monorepo to v4.1.11 ([#906](https://github.com/vnghia/nghe/issues/906)) ([38ed14f](https://github.com/vnghia/nghe/commit/38ed14f7224247816004ca7ea22a9e626ef27b43))
* **deps:** update tailwindcss monorepo to v4.1.12 ([#936](https://github.com/vnghia/nghe/issues/936)) ([d5f260f](https://github.com/vnghia/nghe/commit/d5f260f880ab14d545a5b00db6ead59eaada238b))
* **deps:** update tailwindcss monorepo to v4.1.3 ([#823](https://github.com/vnghia/nghe/issues/823)) ([8eaeeb4](https://github.com/vnghia/nghe/commit/8eaeeb46d4173da1a19ddc57847f797039f998ed))
* **deps:** update tailwindcss monorepo to v4.1.4 ([#834](https://github.com/vnghia/nghe/issues/834)) ([15d54d3](https://github.com/vnghia/nghe/commit/15d54d35ef91036f4abbd6cdfd6713287480688f))
* **deps:** update tailwindcss monorepo to v4.1.5 ([#850](https://github.com/vnghia/nghe/issues/850)) ([45c9637](https://github.com/vnghia/nghe/commit/45c96373e1fe9c964c7d49fa10469eb476f8e896))
* **deps:** update tailwindcss monorepo to v4.1.6 ([#858](https://github.com/vnghia/nghe/issues/858)) ([5cbf1b6](https://github.com/vnghia/nghe/commit/5cbf1b61a226147383f4521c13934194d7d84c8f))
* **deps:** update tailwindcss monorepo to v4.1.7 ([#860](https://github.com/vnghia/nghe/issues/860)) ([5434a1b](https://github.com/vnghia/nghe/commit/5434a1b442586353bc5bb574e60dc6f57564a3fb))
* **deps:** update tailwindcss monorepo to v4.1.8 ([#879](https://github.com/vnghia/nghe/issues/879)) ([921382b](https://github.com/vnghia/nghe/commit/921382b0ba211ab00490c1f62061763be8787473))
* **deps:** update tailwindcss monorepo to v4.1.9 ([#895](https://github.com/vnghia/nghe/issues/895)) ([e851450](https://github.com/vnghia/nghe/commit/e851450c725e3ec26dfea60681bf82e8b89a27d7))
* **lint:** fix lint ([#1065](https://github.com/vnghia/nghe/issues/1065)) ([24ba296](https://github.com/vnghia/nghe/commit/24ba296f22ef87cfbf21f84e353b3c87ce87c29e))
* **nix:** more freely cross configure ([#1070](https://github.com/vnghia/nghe/issues/1070)) ([2a2a11a](https://github.com/vnghia/nghe/commit/2a2a11a83520fe54e47ff6d40edb9a0e26cdbac5))
* remove deprecated dsl from diesel ([#960](https://github.com/vnghia/nghe/issues/960)) ([78f51b0](https://github.com/vnghia/nghe/commit/78f51b020d50d2d401defee96c9ebe8b61a1539e))


### Code Refactoring

* rename nghe-backend to nghe ([#1125](https://github.com/vnghia/nghe/issues/1125)) ([497f49a](https://github.com/vnghia/nghe/commit/497f49a9e6d96f03cdde91150278ce679ea038e5))

## [0.14.1](https://github.com/vnghia/nghe/compare/v0.14.0...v0.14.1) (2026-10-03)


### Bug Fixes

* **ci:** docker tag is wrong for manually triggered ([#1151](https://github.com/vnghia/nghe/issues/1151)) ([c35f17b](https://github.com/vnghia/nghe/commit/c35f17b81c9bd15e88339755f4c5a192a8cd05f7))

## [0.14.0](https://github.com/vnghia/nghe/compare/v0.13.0...v0.14.0) (2026-10-03)


### ⚠ BREAKING CHANGES

* rename nghe-backend to nghe ([#1125](https://github.com/vnghia/nghe/issues/1125))

### Features

* add immutable release ([#1138](https://github.com/vnghia/nghe/issues/1138)) ([70f29ca](https://github.com/vnghia/nghe/commit/70f29ca1a8c25b886decc0787022a0b342f27054))
* support build reproducible release binary ([#1135](https://github.com/vnghia/nghe/issues/1135)) ([4b7b7ba](https://github.com/vnghia/nghe/commit/4b7b7ba6813644f859b1488c5266093ba111c033))


### Code Refactoring

* rename nghe-backend to nghe ([#1125](https://github.com/vnghia/nghe/issues/1125)) ([497f49a](https://github.com/vnghia/nghe/commit/497f49a9e6d96f03cdde91150278ce679ea038e5))

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
