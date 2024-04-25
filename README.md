# Nghe

An implementation of OpenSubsonic API in Rust

- [Nghe](#nghe)
  - [Features](#features)
  - [Getting started](#getting-started)
  - [Client](#client)
  - [Configuration](#configuration)
    - [Server](#server)
    - [Database](#database)
    - [Artist](#artist)
    - [Parsing](#parsing)
      - [Song](#song)
      - [Album](#album)
      - [Id3v2](#id3v2)
      - [Number and total](#number-and-total)
    - [Scan](#scan)
    - [Transcoding](#transcoding)
    - [Art](#art)
  - [Scan process](#scan-process)
    - [How a song is uniquely identified ?](#how-a-song-is-uniquely-identified--)
    - [Scan mode](#scan-mode)
      - [Full](#full)
      - [Force](#force)
    - [How an artist is uniquely identified ?](#how-an-artist-is-uniquely-identified--)
    - [How an album is uniquely identified ?](#how-an-album-is-uniquely-identified--)
  - [Permission model](#permission-model)
    - [Access to a song-level resource](#access-to-a-song-level-resource)
    - [Access to an artist or album level resource](#access-to-an-artist-or-album-level-resource)
  - [Roadmap](#roadmap)

## Features

- Written in Rust with performance in-mind.
- All tags are multiple by default (artists, album artists, languages, etc).
- Well-tested and highly customizable.
- Well-defined permission model with music folders.
- Multi-platform, runs on Linux, FreeBSD, MacOS and Windows. Docker images with two variants GNU or MUSL are also provided.
- Bridging with `ffmpeg c api` for in-memory transcoding and smooth stream experience. Most common formats (opus, mp3, acc, wav, etc) are supported. Does not required any manual configuration beforehand, just `maxBitRate` and `format` in the request parameters are enough.
- Synchoronized lyrics from external `lrc` files.

## Getting started

The easiest way for getting started is using docker. Below is a `docker-compose.yaml` example. A random hex key can be generated using `openssl rand -hex 16`.

```yaml
services:
  nghe:
    image: ghcr.io/vnghia/nghe-musl:0.3.0
    ports:
      - "3000:3000"
    restart: unless-stopped
    environment:
      NGHE_DATABASE__URL: postgres://postgres:postgres@db:5432/postgres
      NGHE_DATABASE__KEY: a20eb15ac92cabfd96b81fb154b16357
    volumes:
      - /your-music-folder/:/data/music/:ro
    depends_on:
      db:
        condition: service_healthy

  db:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: postgres
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5
```

The minimum required postgres version is 15. Alternatively, you can also download corresponding binary with your OS and the frontend package in the release section.

Once the server is running, go to `[your-server-url]/setup` to setup a first admin account. After that, login and go to the folders menu on the right side of the screen. You can add a new music folder from there, hit the scan button, choose one scan mode (more detail in [scan process](#scan-process)) and start using the server while your media files are being scanned.

## Client

This server works best with [Symfonium](https://symfonium.app/) and [Airsonic](https://airsonic.netlify.app) since they support multiple values well enough.

## Configuration

All configurations can be set by environment variable with a `NGHE_` prefix and a `__` between each level of inheritance. For example, the config `database.url` is correspondent to `NGHE_DATABASE__URL`.

### Server

|    Subkey    | Meaning                                       | Default value         | Note                        |
| :----------: | :-------------------------------------------- | :-------------------- | :-------------------------- |
|     host     | IP host to bind the server to                 | `127.0.0.1`           | `::` for Docker             |
|     port     | Port to bind the server to                    | 3000                  |                             |
| frontend_dir | The directory contains the pre-built frontend | `$PWD/frontend/dist/` | `/app/frontend/` for Docker |

### Database

| Subkey | Meaning                                                                           | Default value | Note |
| :----: | :-------------------------------------------------------------------------------- | :------------ | :--- |
|  url   | URL to connect to the database                                                    |               |      |
|  key   | A 32-characters hex string to use as encryption key for sensetive data (password) |               |      |

### Artist

|      Subkey      | Meaning                                          | Default value                         | Note |
| :--------------: | :----------------------------------------------- | :------------------------------------ | :--- |
| ignored_articles | Articles to ignore while building artist indexes | "The An A Die Das Ein Eine Les Le La" |      |

### Parsing

Parsing config allows you to tells the server exactly where to read your songs metadata. Currently, belows tags are supported.

|    Subkey     | Meaning                            | [Id3v2](#id3v2)               | VorbisComments            | Note                                                                                                                   |
| :-----------: | :--------------------------------- | :---------------------------- | :------------------------ | :--------------------------------------------------------------------------------------------------------------------- |
|     song      | Subconfiguration for parsing song  |                               |                           | [song](#song)                                                                                                          |
|     album     | Subconfiguration for parsing album |                               |                           | [album](#album)                                                                                                        |
|    artist     | Artist names                       | TPE1                          | ARTIST                    |                                                                                                                        |
| album_artist  | Album artist names                 | TPE2                          | ALBUMARTIST               |                                                                                                                        |
| track_number  | Track number                       | TRCK                          | TRACKNUMBER               | [number and total](#number-and-total)                                                                                  |
|  track_total  | Track Total                        |                               | TRACKTOTAL                | [number and total](#number-and-total)                                                                                  |
|  disc_number  | Disc number                        | TPOS                          | DISCNUMBER                | [number and total](#number-and-total)                                                                                  |
|  disc_total   | Disc Total                         |                               | DISCTOTAL                 | [number and total](#number-and-total)                                                                                  |
|   language    | Languages                          | TLAN                          | LANGUAGE                  | Should be a ISO 639-3 or 639-1 code, **not** 639-2                                                                     |
|     genre     | Genres                             | TCON                          | GENRE                     |                                                                                                                        |
| artist_mbz_id | Artist musicbrainz id              | "MusicBrainz Artist Id"       | MUSICBRAINZ_ARTISTID      | Should be specified only if you have only one artist in the tag because the order is not perserved while parsing       |
| artist_mbz_id | Artist musicbrainz id              | "MusicBrainz Album Artist Id" | MUSICBRAINZ_ALBUMARTISTID | Should be specified only if you have only one album artist in the tag because the order is not perserved while parsing |

#### Song

Some fields in this table are not official and usually used to distinguish between date-related information of albums and songs. If you have don't have that information, just leave it as default.

|        Subkey         | Meaning                    | Id3v2                          | VorbisComments             | Note                                                |
| :-------------------: | :------------------------- | :----------------------------- | :------------------------- | :-------------------------------------------------- |
|         name          | Song name                  | TIT2                           | TITLE                      |                                                     |
|         date          | Song recording date        | TRCS                           | SDATE                      | Set `null` to completely disable parsing this field |
|     release_date      | Song release date          | TSRL                           | SRELEASEDATE               | Set `null` to completely disable parsing this field |
| original_release_date | Song original release date | TSOR                           | SORIGYEAR                  | Set `null` to completely disable parsing this field |
|        mbz_id         | Song musicbrainz id        | "MusicBrainz Release Track Id" | MUSICBRAINZ_RELEASETRACKID |                                                     |

#### Album

|        Subkey         | Meaning                     | Id3v2                  | VorbisComments      | Note                                                |
| :-------------------: | :-------------------------- | :--------------------- | :------------------ | :-------------------------------------------------- |
|         name          | Album name                  | TALB                   | ALBUM               |                                                     |
|         date          | Album recording date        | TDRC                   | DATE                | Set `null` to completely disable parsing this field |
|     release_date      | Album release date          | TDRL                   | RELEASEDATE         | Set `null` to completely disable parsing this field |
| original_release_date | Album original release date | TDOR                   | ORIGYEAR            | Set `null` to completely disable parsing this field |
|        mbz_id         | Album musicbrainz id        | "MusicBrainz Album Id" | MUSICBRAINZ_ALBUMID |                                                     |

#### Id3v2

Id3v2 key is treated as two different ways depending on its length:

- If you supply a 3 or 4 characters string, it will be treated as a frame id. For example TIT2.
- Otherwise, it will be treated as an user text key in the frame TXXX. For example "MusicBrainz Release Track Id".

In additional to those configurations above, id3v2 also has below configuration.

|  Subkey   | Meaning                                  | Default value | Note                                                                                                             |
| :-------: | :--------------------------------------- | :------------ | :--------------------------------------------------------------------------------------------------------------- |
| separator | Separator for multiple values in id3v2.3 | '/'           | This is only applicable with id3v2.3, id3v2.4 will always be splited by '\0' (or '\\\\' if you are using mp3tag) |

#### Number and total

You can set track/disc number and total by three ways below:

- Setting fields `track_number`, `track_total`, etc normally. Unfortunately this does not work on id3v2.
- Using only `track_number` and set it to `{track_number}/{track_total}`. This sets `track_number` and `track_total` to the corresponding numbers after spliting and works with all formats. The same thing holds for `disc_number`.
- Using only `track_number` and set it to `{character}{track_number}` like `A1`, `B10`. This sets the `disc_number` to the order of that character in the alphabet and `track_number` to the following number. `track_total` and `disc_total` will be none. This format is encountered while parsing vinyl records.

### Scan

Scan process has two main threads, one thread (**the walking thread**) will be responsible for walking directories in the filesystem and send back the result to the second thread (**the parsing thread**), who is responsible for parsing each file and updating information in the database. More in [scan process](#scan-process).

You should tweak this carefully to find the optimized value for your system. Sometimes, running concurrently is not the fastest option.

|    Subkey    | Meaning                                                                                                                                                                        | Default value | Note |
| :----------: | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------ | :--- |
|   parallel   | If the walking thread should spawn more threads and run concurrently.                                                                                                          | false         |      |
| channel_size | The maximum number of results that can be sent back to the parsing thread. If the results queue is full, the walking threads will be blocked until the queue has an empty slot | 10            |      |
|  pool_size   | The maximum number of threads that the parsing thread can spawn to process the result                                                                                          | 10            |      |

### Transcoding

|   Subkey    | Meaning                                          | Default value                    | Note                                                                    |
| :---------: | :----------------------------------------------- | :------------------------------- | :---------------------------------------------------------------------- |
| buffer_size | Buffer size to allocate for custom `AVIOContext` | 32 \* 1024                       |                                                                         |
| cache_path  | The cache directory to save transcoding results  | `$TMPDIR/nghe/cache/transcoding` | Set `null` or a non-absolute path to completely disable parsing caching |

### Art

|   Subkey   | Meaning                                 | Default value             | Note                                                                           |
| :--------: | :-------------------------------------- | :------------------------ | :----------------------------------------------------------------------------- |
| artist_dir | The directory to save artist cover arts | `$TMPDIR/nghe/art/artist` | Set `null` or a non-absolute path to completely disable song cover art extract |
|  song_dir  | The directory to save song cover arts   | `$TMPDIR/nghe/art/song`   | Set `null` or a non-absolute path to completely disable song cover art extract |

### Lastfm

| Subkey | Meaning                         | Default value | Note |
| :----: | :------------------------------ | :------------ | :--- |
|  key   | Lastfm key to fetch information |               |      |

### Spotify

| Subkey | Meaning                                    | Default value | Note |
| :----: | :----------------------------------------- | :------------ | :--- |
|   id   | Spotify client id to fetch information     |               |      |
| secret | Spotify client secret to fetch information |               |      |

## Scan process

### How a song is uniquely identified ?

A song is uniquely identified either by:

- Its music folder and relative path (to its music folder).
- Its music folder, hash and size. It means that you can not have duplicated songs in the same music folder. If a duplicated songs is detected, the path will be updated to the latest encountered path. It allows you to move freely your songs in the same music folder, only its path will be updated in the database.

Duplication on two different music folders are is allowed and will be treated as two seperate songs.

### Scan mode

#### Full

As describe above, if an already identified song is scanned (by checking relative path, hash and size), the scanning process will just mark the file as scanned and skip that file. In the end of the scanning process, all old songs and related informations (artists/albums/genres/etc) inside that music folder that are not scanned will be deleted.

#### Force

Same as full but will try parsing the file regardless if it is identified or not. This mode is useful if there are new metadata that added into the scanning process.

### How an artist is uniquely identified ?

An artist is uniquely identified either by:

- Their musicbrainz id
- Their name if their musicbrainz id is empty.

If you have duplicated artists shown in your client, you should check for the corresponding musicbrainz id field.

### How an album is uniquely identified ?

An album is uniquely identified either by:

- Their musicbrainz id
- Their name, date, release date, original release date if their musicbrainz id is empty.

If you have duplicated albums shown in your client, you should check for the corresponding musicbrainz id and date fields.

## Permission model

As describe in [scan process](#scan-process), songs are tied to a specific music folder but artists and albums are globally identified. There will be two scenarios.

### Access to a song-level resource

In this case, user is either allowed or denied depending on their allowed music folders. Song-level resource is the song itself, the lyrics, the cover art.

### Access to an artist or album level resource

In this case, user will only see a combination of allowed song-level resources, not the whole thing.

For example, if an artist has **10** songs in **folder1** and **20** songs in **folder2**:

- User with permission on both folders will see that artist has 30 songs.
- User with permission on folder1 will only see first 10 songs.
- User with permission on folder2 will only see last 20 songs.
- User with no permission on both folder will not see the artist in the first place and get a not found error if they are trying to access it by manully specifying the id.

The same thing holds true for albums. For example, if an album has 2 songs in 2 folders with different cover arts:

- User with permission on both folders will see the first cover art sorted by the smallest disc number and track number (track 1 disc 1 of the album).
- User with permission on folder1 will only see the first cover art.
- User with permission on folder2 will only see the second cover art.

### Nested music folder

Nested music folders are allowed and each folder will be treated as a separate unrelated folder.

### Configuration

Permission configuration can be found in the folders menu of the frontend.

## Roadmap

- More compatible with Opensubsonic API.
- Fully-feature frontend.
- SQL playlist builder.
- Possibly integrating local machine learning model (like how Immich is doing with image/videos).
