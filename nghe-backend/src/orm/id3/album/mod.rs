pub mod id_duration;
pub mod with_artists_songs;

use diesel::prelude::*;
use nghe_api::id3;
use nghe_api::id3::builder::album as builder;
use uuid::Uuid;

use super::genre::Genres;
use crate::orm::{albums, songs};
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
pub struct Album {
    pub id: Uuid,
    pub name: String,
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

pub type BuilderSet = builder::SetReleaseDate<
    builder::SetOriginalReleaseDate<
        builder::SetGenres<
            builder::SetMusicBrainzId<builder::SetYear<builder::SetName<builder::SetId>>>,
        >,
    >,
>;

impl Album {
    pub fn try_into_api_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        Ok(id3::album::Album::builder()
            .id(self.id)
            .name(self.name)
            .maybe_year(self.date.year.map(i16::try_into).transpose()?)
            .maybe_music_brainz_id(self.music_brainz_id)
            .genres(self.genres.into())
            .original_release_date(self.original_release_date.try_into()?)
            .release_date(self.release_date.try_into()?))
    }
}

pub mod query {
    use diesel::dsl::auto_type;

    use super::*;
    use crate::orm::{genres, songs_genres};

    #[auto_type]
    pub fn unchecked_no_group_by() -> _ {
        albums::table
            .inner_join(songs::table)
            .left_join(songs_genres::table.on(songs_genres::song_id.eq(songs::id)))
            .left_join(genres::table.on(genres::id.eq(songs_genres::genre_id)))
            .order_by(albums::name)
    }

    #[auto_type]
    pub fn unchecked() -> _ {
        unchecked_no_group_by().group_by(albums::id)
    }
}
