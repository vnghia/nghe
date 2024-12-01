pub mod durations;
pub mod full;
pub mod short;

use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::common::format::Trait as _;
use nghe_api::id3;
use nghe_api::id3::builder::song as builder;
use time::OffsetDateTime;
use uuid::Uuid;

use super::artist;
use crate::file::audio;
use crate::file::audio::duration::Trait as _;
use crate::orm::songs;
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    #[diesel(embed)]
    pub track: songs::position::Track,
    #[diesel(embed)]
    pub date: songs::date::Date,
    #[diesel(select_expression = sql(
        "coalesce(songs.cover_art_id, any_value(albums.cover_art_id)) cover_art_id"
    ))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Nullable<sql_types::Uuid>>)]
    pub cover_art: Option<Uuid>,
    #[diesel(embed)]
    pub file: songs::property::File,
    #[diesel(embed)]
    pub property: songs::property::Property,
    #[diesel(embed)]
    pub disc: songs::position::Disc,
    #[diesel(column_name = created_at)]
    pub created: OffsetDateTime,
    #[diesel(embed)]
    pub artists: artist::required::Artists,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
}

pub type BuilderSet = builder::SetMusicBrainzId<
    builder::SetArtists<
        builder::SetCreated<
            builder::SetDiscNumber<
                builder::SetChannelCount<
                    builder::SetSamplingRate<
                        builder::SetBitDepth<
                            builder::SetBitRate<
                                builder::SetDuration<
                                    builder::SetSuffix<
                                        builder::SetContentType<
                                            builder::SetSize<
                                                builder::SetCoverArt<
                                                    builder::SetYear<
                                                        builder::SetTrack<
                                                            builder::SetTitle<builder::SetId>,
                                                        >,
                                                    >,
                                                >,
                                            >,
                                        >,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
>;

impl audio::duration::Trait for Song {
    fn duration(&self) -> audio::Duration {
        self.property.duration.duration()
    }
}

impl Song {
    pub fn try_into_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        let duration = self.duration();
        Ok(id3::song::Song::builder()
            .id(self.id)
            .title(self.title)
            .track(self.track.number.map(u16::try_from).transpose()?)
            .year(self.date.year.map(u16::try_from).transpose()?)
            .cover_art(self.cover_art)
            .size(self.file.size.cast_unsigned())
            .content_type(self.file.format.mime().into())
            .suffix(self.file.format.extension().into())
            .duration(duration.into())
            .bit_rate(self.property.bitrate.try_into()?)
            .bit_depth(self.property.bit_depth.map(u8::try_from).transpose()?)
            .sampling_rate(self.property.sample_rate.try_into()?)
            .channel_count(self.property.channel_count.try_into()?)
            .disc_number(self.disc.number.map(u16::try_from).transpose()?)
            .created(self.created)
            .artists(self.artists.into())
            .music_brainz_id(self.music_brainz_id))
    }
}

impl TryFrom<Song> for id3::song::Song {
    type Error = Error;

    fn try_from(value: Song) -> Result<Self, Self::Error> {
        Ok(value.try_into_builder()?.build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::{albums, songs_artists, star_songs};

    #[auto_type]
    pub fn unchecked_no_group_by() -> _ {
        songs::table
            .inner_join(songs_artists::table)
            .inner_join(albums::table)
            .inner_join(artist::required::query::song())
            .left_join(star_songs::table)
            .order_by((
                songs::disc_number.asc().nulls_first(),
                songs::track_number.asc().nulls_first(),
                songs::title.asc(),
            ))
    }

    #[auto_type]
    pub fn unchecked() -> _ {
        let song: AsSelect<Song, crate::orm::Type> = Song::as_select();
        unchecked_no_group_by().group_by(songs::id).select(song)
    }
}

#[cfg(test)]
mod test {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::picture;
    use crate::orm::songs;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;

        music_folder.add_audio_artist(["1".into(), "2".into()], [Faker.fake()], false, 1).await;
        let song_id = music_folder.song_id(0);

        let database_song = query::unchecked()
            .filter(songs::id.eq(song_id))
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        let artists: Vec<String> = database_song.artists.into();
        assert_eq!(artists, &["1", "2"]);
    }

    #[rstest]
    #[tokio::test]
    async fn test_query_cover_art(
        #[future(awt)] mock: Mock,
        #[values(true, false)] has_picture: bool,
        #[values(true, false)] has_dir_picture: bool,
    ) {
        let mut music_folder = mock.music_folder(0).await;

        let (picture, picture_id) = if has_picture {
            let picture: picture::Picture = Faker.fake();
            let picture_id = picture.upsert_mock(&mock).await;
            (Some(picture), Some(picture_id))
        } else {
            (None, None)
        };

        let (dir_picture, dir_picture_id) = if has_dir_picture {
            let dir_picture: picture::Picture = Faker.fake();
            let source = music_folder.path().join(dir_picture.property.format.name()).to_string();
            let dir_picture = dir_picture.with_source(Some(source));
            let dir_picture_id = dir_picture.upsert_mock(&mock).await;
            (Some(dir_picture), Some(dir_picture_id))
        } else {
            (None, None)
        };

        music_folder
            .add_audio_filesystem::<&str>()
            .album(Faker.fake())
            .picture(picture)
            .dir_picture(dir_picture)
            .depth(0)
            .n_song(10)
            .call()
            .await;

        let songs = query::unchecked().get_results(&mut mock.get().await).await.unwrap();
        for song in songs {
            if has_picture {
                assert_eq!(song.cover_art, picture_id);
            } else if has_dir_picture {
                assert_eq!(song.cover_art, dir_picture_id);
            } else {
                assert!(song.cover_art.is_none());
            }
        }
    }
}
