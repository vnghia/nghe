use lofty::flac::FlacFile;
use lofty::id3::v2::Id3v2Tag;
use lofty::mpeg::MpegFile;
use lofty::ogg::{OggPictureStorage as _, VorbisComments};

use super::Metadata;
use crate::config;
use crate::file::audio::{Album, Artists, Genres, NameDateMbz, TrackDisc};
use crate::file::image::Image;
use crate::file::lyric::Lyric;

trait TagMut {
    type Tag: Metadata;
    fn tag_mut(&mut self) -> &mut Self::Tag;
}

default impl<T: TagMut> Metadata for T {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self {
        self.tag_mut().dump_song(config, song);
        self
    }

    fn dump_album(&mut self, config: &config::Parsing, album: Album<'_>) -> &mut Self {
        self.tag_mut().dump_album(config, album);
        self
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self {
        self.tag_mut().dump_artists(config, artists);
        self
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self {
        self.tag_mut().dump_track_disc(config, track_disc);
        self
    }

    fn dump_languages(
        &mut self,
        config: &config::Parsing,
        languages: Vec<isolang::Language>,
    ) -> &mut Self {
        self.tag_mut().dump_languages(config, languages);
        self
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Genres<'_>) -> &mut Self {
        self.tag_mut().dump_genres(config, genres);
        self
    }

    fn dump_lyrics(&mut self, config: &config::Parsing, lyrics: Vec<Lyric<'_>>) -> &mut Self {
        self.tag_mut().dump_lyrics(config, lyrics);
        self
    }

    fn dump_image(&mut self, image: Option<Image<'_>>) -> &mut Self {
        self.tag_mut().dump_image(image);
        self
    }
}

impl TagMut for FlacFile {
    type Tag = VorbisComments;
    fn tag_mut(&mut self) -> &mut VorbisComments {
        self.vorbis_comments_mut().unwrap()
    }
}

impl Metadata for FlacFile {
    fn dump_image(&mut self, image: Option<Image<'_>>) -> &mut Self {
        if let Some(image) = image {
            self.insert_picture(image.into(), None).unwrap();
        }
        self
    }
}

impl TagMut for MpegFile {
    type Tag = Id3v2Tag;
    fn tag_mut(&mut self) -> &mut Id3v2Tag {
        self.id3v2_mut().unwrap()
    }
}

impl Metadata for MpegFile {}
