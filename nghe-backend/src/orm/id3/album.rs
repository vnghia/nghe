use diesel::dsl::{count, sum, AssumeNotNull};
use diesel::helper_types;
use diesel::prelude::*;
use nghe_api::id3;
use nghe_api::id3::builder::album as builder;
use num_traits::ToPrimitive as _;
use uuid::Uuid;

use super::genre::Genres;
use crate::orm::{albums, songs};
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    #[diesel(select_expression = count(songs::id))]
    pub song_count: i64,
    #[diesel(select_expression = sum(songs::duration).assume_not_null())]
    #[diesel(select_expression_type = AssumeNotNull<helper_types::sum<songs::duration>>)]
    pub duration: f32,
    #[diesel(embed)]
    pub date: albums::date::Date,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
    #[diesel(embed)]
    pub genres: Genres,
    #[diesel(embed)]
    pub original_release_date: albums::date::OriginalRelease,
    #[diesel(embed)]
    pub release_date: albums::date::Release,
}

pub type AlbumBuilderSet = builder::SetReleaseDate<
    builder::SetOriginalReleaseDate<
        builder::SetGenres<
            builder::SetMusicBrainzId<
                builder::SetYear<
                    builder::SetDuration<builder::SetSongCount<builder::SetName<builder::SetId>>>,
                >,
            >,
        >,
    >,
>;

impl Album {
    pub fn try_into_api_builder(self) -> Result<builder::Builder<AlbumBuilderSet>, Error> {
        Ok(id3::Album::builder()
            .id(self.id)
            .name(self.name)
            .song_count(self.song_count.try_into()?)
            .duration(
                self.duration
                    .ceil()
                    .to_u32()
                    .ok_or_else(|| Error::CouldNotConvertFloatToInteger(self.duration))?,
            )
            .maybe_year(self.date.year.map(i16::try_into).transpose()?)
            .maybe_music_brainz_id(self.music_brainz_id)
            .genres(self.genres.into())
            .original_release_date(self.original_release_date.try_into()?)
            .release_date(self.release_date.try_into()?))
    }

    pub fn try_into_api(self) -> Result<id3::Album, Error> {
        Ok(self.try_into_api_builder()?.build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::{genres, songs_genres};

    #[auto_type]
    pub fn unchecked() -> _ {
        let album: AsSelect<Album, crate::orm::Type> = Album::as_select();
        albums::table
            .inner_join(songs::table)
            .inner_join(songs_genres::table.on(songs_genres::song_id.eq(songs::id)))
            .inner_join(genres::table.on(genres::id.eq(songs_genres::genre_id)))
            .group_by(albums::id)
            .order_by(albums::name)
            .select(album)
    }
}
