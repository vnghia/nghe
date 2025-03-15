pub mod artists;
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

use crate::file::audio;
use crate::file::audio::duration::Trait as _;
use crate::orm::songs;
use crate::{Error, error};

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
    pub artists: artists::Artists,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
    #[diesel(select_expression = sql("any_value(star_songs.created_at) starred"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Nullable<sql_types::Timestamptz>>)]
    pub starred: Option<OffsetDateTime>,
}

pub type BuilderSet = builder::SetStarred<
    builder::SetMusicBrainzId<
        builder::SetArtists<
            builder::SetArtistId<
                builder::SetArtist<
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
                                                                        builder::SetTitle<
                                                                            builder::SetId,
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
        let main_artist =
            self.artists.value.first().ok_or_else(|| error::Kind::DatabaseCorruptionDetected)?;
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
            .artist(main_artist.name.clone())
            .artist_id(main_artist.id)
            .artists(self.artists.into())
            .music_brainz_id(self.music_brainz_id)
            .starred(self.starred))
    }
}

impl TryFrom<Song> for id3::song::Song {
    type Error = Error;

    fn try_from(value: Song) -> Result<Self, Self::Error> {
        Ok(value.try_into_builder()?.build())
    }
}

impl Song {
    pub fn try_into_short(self, album: String, album_id: Uuid) -> Result<id3::song::Short, Error> {
        Ok(id3::song::Short { song: self.try_into_builder()?.build(), album, album_id })
    }
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::id3::artist;
    use crate::orm::{albums, songs_artists, star_songs};

    #[auto_type]
    pub fn with_user_id_unchecked_no_group_by(user_id: Uuid) -> _ {
        songs::table
            .inner_join(songs_artists::table)
            .inner_join(albums::table)
            .inner_join(artist::required::query::song())
            .left_join(
                star_songs::table
                    .on(star_songs::song_id.eq(songs::id).and(star_songs::user_id.eq(user_id))),
            )
            .order_by((
                songs::disc_number.asc().nulls_first(),
                songs::track_number.asc().nulls_first(),
                songs::title.asc(),
            ))
    }

    #[auto_type]
    pub fn with_user_id_unchecked(user_id: Uuid) -> _ {
        let with_user_id_unchecked_no_group_by: with_user_id_unchecked_no_group_by =
            with_user_id_unchecked_no_group_by(user_id);
        let song: AsSelect<Song, crate::orm::Type> = Song::as_select();
        with_user_id_unchecked_no_group_by.group_by(songs::id).select(song)
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::image;
    use crate::orm::songs;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;
        let artist_names = fake::vec![String; 2..4];
        music_folder
            .add_audio_artist(
                artist_names.clone().into_iter().map(std::convert::Into::into),
                [Faker.fake()],
                false,
                1,
            )
            .await;
        let song_id = music_folder.song_id(0);

        let database_song = query::with_user_id_unchecked(mock.user_id(0).await)
            .filter(songs::id.eq(song_id))
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        let artists: Vec<String> = database_song.artists.into();
        assert_eq!(artists, artist_names);
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
            let picture: image::Image = Faker.fake();
            let picture_id = picture.upsert_mock(&mock, None::<&str>).await;
            (Some(picture), Some(picture_id))
        } else {
            (None, None)
        };

        let (dir_picture, dir_picture_id) = if has_dir_picture {
            let dir_picture: image::Image = Faker.fake();
            let dir_picture_id = dir_picture
                .upsert_mock(
                    &mock,
                    Some(music_folder.path().join(dir_picture.property.format.name()).to_string()),
                )
                .await;
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

        let songs = query::with_user_id_unchecked(mock.user_id(0).await)
            .get_results(&mut mock.get().await)
            .await
            .unwrap();
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
