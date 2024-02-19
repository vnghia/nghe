use diesel::query_source::{Alias, AliasedField};

diesel::alias!(super::songs as songs: Songs);
diesel::alias!(super::songs_artists as songs_artists: SongsArtists);
diesel::alias!(super::songs_album_artists as songs_album_artists: SongsAlbumArtists);

pub type SongsTable = Alias<Songs>;

pub type SongsId = AliasedField<Songs, super::songs::id>;
pub type SongsAlbumId = AliasedField<Songs, super::songs::album_id>;
pub type SongsMusicFolderId = AliasedField<Songs, super::songs::music_folder_id>;

pub type SongsArtistsTable = Alias<SongsArtists>;

pub type SongsArtistsSongId = AliasedField<SongsArtists, super::songs_artists::song_id>;
pub type SongsArtistsArtistId = AliasedField<SongsArtists, super::songs_artists::artist_id>;

pub type SongsAlbumArtistsTable = Alias<SongsAlbumArtists>;

pub type SongsAlbumArtistsSongId =
    AliasedField<SongsAlbumArtists, super::songs_album_artists::song_id>;
pub type SongsAlbumArtistsAlbumArtistId =
    AliasedField<SongsAlbumArtists, super::songs_album_artists::album_artist_id>;
