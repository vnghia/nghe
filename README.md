# Nghe

An implementation of OpenSubsonic API in Rust

## Features

- Written in Rust with performance in-mind.
- All tags are multiple by default (artists, album artists, languages, etc).
- Well-tested and highly customizable.
- Well-defined permission model with music folders.
- Multi-platform, runs on macOS, Linux and Windows. Docker images are also provided.
- Bridging with `ffmpeg c api` for in-memory transcoding and smooth stream experience. Most common formats (opus, mp3, acc, wav, etc) are supported. Does not required any manual configuration beforehand, just `maxBitRate` and `format` in the request parameters are enough.

## Music folders

The server accepts multiple music folders and the admin can set permission of these music folders for each user / folder. If there is an artist / album has songs in several music folders, user will only get what are inside their allowed music folders.
