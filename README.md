# Nghe

An implementation of OpenSubsonic API in Rust

## Features

- Written in Rust with performance in-mind.
- All tags are multiple by default (artists, album artists, languages, etc).
- Well-tested and highly customizable.
- Well-defined permission model with music folders.
- Multi-platform, runs on macOS, Linux and Windows. Docker images are also provided.
- Bridging with `ffmpeg c api` for in-memory transcoding and smooth stream experience. Most common formats (opus, mp3, acc, wav, etc) are supported. Does not required any manual configuration beforehand, just `maxBitRate` and `format` in the request parameters are enough.

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

In additional to those configurations above, `id3v2` also has below configuration.

|  Subkey   | Meaning                                  | Default value | Note                                                                                                             |
| :-------: | :--------------------------------------- | :------------ | :--------------------------------------------------------------------------------------------------------------- |
| separator | Separator for multiple values in id3v2.3 | '/'           | This is only applicable with id3v2.3, id3v2.4 will always be splited by '\0' (or '\\\\' if you are using mp3tag) |

#### Number and total

You can set track/disc number and total by three ways below:

- Setting fields `track_number`, `track_total`, etc normally. Unfortunately this does not work on id3v2.
- Using only `track_number` and set it to `{track_number}/{track_total}`. This sets `track_number` and `track_total` to the corresponding numbers after spliting and works with all formats. The same thing holds for `disc_number`.
- Using only `track_number` and set it to `{character}{track_number}` like `A1`, `B10`. This sets the `disc_number` to the order of that character in the alphabet and `track_number` to the following number. `track_total` and `disc_total` will be none. This format is encountered while parsing vinyl records.

### Scan

Scan process has two main threads, one thread (**the walking thread**) will be responsible for walking directories in the filesystem and send back the result to the second thread (**the parsing thread**), who is responsible for parsing each file and updating information in the database.

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

|  Subkey   | Meaning                               | Default value           | Note                                                                           |
| :-------: | :------------------------------------ | :---------------------- | :----------------------------------------------------------------------------- |
| song_path | The directory to save song cover arts | `$TMPDIR/nghe/art/song` | Set `null` or a non-absolute path to completely disable song cover art extract |

## Music folders

The server accepts multiple music folders and the admin can set permission of these music folders for each user / folder. If there is an artist / album has songs in several music folders, user will only get what are inside their allowed music folders.
