pub mod durations;
pub mod with_album_genres;

use diesel::prelude::*;
use nghe_api::common::format::Trait as _;
use nghe_api::id3;
use nghe_api::id3::builder::song as builder;
use uuid::Uuid;

use super::artist;
use super::duration::Trait as _;
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
    #[diesel(embed)]
    pub file: songs::property::File,
    #[diesel(embed)]
    pub property: songs::property::Property,
    #[diesel(embed)]
    pub disc: songs::position::Disc,
    #[diesel(embed)]
    pub artists: artist::required::Artists,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
}

pub type BuilderSet = builder::SetMusicBrainzId<
    builder::SetArtists<
        builder::SetDiscNumber<
            builder::SetChannelCount<
                builder::SetSamplingRate<
                    builder::SetBitDepth<
                        builder::SetBitRate<
                            builder::SetDuration<
                                builder::SetSuffix<
                                    builder::SetContentType<
                                        builder::SetSize<
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
>;

impl Song {
    pub fn try_into_api_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        let duration = self.duration()?;
        Ok(id3::song::Song::builder()
            .id(self.id)
            .title(self.title)
            .maybe_track(self.track.number.map(u16::try_from).transpose()?)
            .maybe_year(self.date.year.map(u16::try_from).transpose()?)
            .size(self.file.size.cast_unsigned())
            .content_type(self.file.format.mime())
            .suffix(self.file.format.extension())
            .duration(duration)
            .bit_rate(self.property.bitrate.try_into()?)
            .maybe_bit_depth(self.property.bit_depth.map(u8::try_from).transpose()?)
            .sampling_rate(self.property.sample_rate.try_into()?)
            .channel_count(self.property.channel_count.try_into()?)
            .maybe_disc_number(self.disc.number.map(u16::try_from).transpose()?)
            .artists(self.artists.into())
            .maybe_music_brainz_id(self.music_brainz_id))
    }

    pub fn try_into_api(self) -> Result<id3::song::Song, Error> {
        Ok(self.try_into_api_builder()?.build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::songs_artists;

    #[auto_type]
    pub fn unchecked_no_group_by() -> _ {
        songs::table
            .inner_join(songs_artists::table)
            .inner_join(artist::required::query::song())
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
}
