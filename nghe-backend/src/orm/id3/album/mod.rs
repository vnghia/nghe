pub mod full;
pub mod short;

use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
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
    #[diesel(select_expression = sql(
        "(array_agg(songs.cover_art_id order by \
        songs.disc_number asc nulls last, songs.track_number \
        asc nulls last, songs.cover_art_id asc) filter \
        (where songs.cover_art_id is not null))[1] cover_art_id"
    ))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Nullable<sql_types::Uuid>>)]
    pub cover_art: Option<Uuid>,
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
            builder::SetMusicBrainzId<
                builder::SetYear<builder::SetCoverArt<builder::SetName<builder::SetId>>>,
            >,
        >,
    >,
>;

impl Album {
    pub fn try_into_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        Ok(id3::album::Album::builder()
            .id(self.id)
            .name(self.name)
            .cover_art(self.cover_art)
            .year(self.date.year.map(u16::try_from).transpose()?)
            .music_brainz_id(self.music_brainz_id)
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
