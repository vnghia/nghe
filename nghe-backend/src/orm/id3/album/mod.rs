pub mod artists;
pub mod full;
pub mod short;

use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use nghe_api::id3::builder::album as builder;
use time::OffsetDateTime;
use uuid::Uuid;

use super::genre::Genres;
use crate::Error;
use crate::orm::{albums, songs};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    #[diesel(select_expression = sql(
        "coalesce(albums.cover_art_id, (array_remove(array_agg(songs.cover_art_id \
        order by songs.disc_number asc nulls last, songs.track_number asc \
        nulls last, songs.cover_art_id asc), null))[1]) cover_art_id"
    ))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Nullable<sql_types::Uuid>>)]
    pub cover_art: Option<Uuid>,
    #[diesel(column_name = created_at)]
    pub created: OffsetDateTime,
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
    #[diesel(select_expression = sql("any_value(star_albums.created_at) starred"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Nullable<sql_types::Timestamptz>>)]
    pub starred: Option<OffsetDateTime>,
}

pub type BuilderSet = builder::SetStarred<
    builder::SetReleaseDate<
        builder::SetOriginalReleaseDate<
            builder::SetGenres<
                builder::SetMusicBrainzId<
                    builder::SetYear<
                        builder::SetCreated<builder::SetCoverArt<builder::SetName<builder::SetId>>>,
                    >,
                >,
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
            .created(self.created)
            .year(self.date.year.map(u16::try_from).transpose()?)
            .music_brainz_id(self.music_brainz_id)
            .genres(self.genres.into())
            .original_release_date(self.original_release_date.try_into()?)
            .release_date(self.release_date.try_into()?)
            .starred(self.starred))
    }
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::{genres, songs_genres, star_albums};

    #[auto_type]
    pub fn with_user_id_unchecked_no_group_by(user_id: Uuid) -> _ {
        albums::table
            .inner_join(songs::table)
            .left_join(songs_genres::table.on(songs_genres::song_id.eq(songs::id)))
            .left_join(genres::table.on(genres::id.eq(songs_genres::genre_id)))
            .left_join(
                star_albums::table
                    .on(star_albums::album_id.eq(albums::id).and(star_albums::user_id.eq(user_id))),
            )
            .order_by(albums::name)
    }

    #[auto_type]
    pub fn with_user_id_unchecked(user_id: Uuid) -> _ {
        let with_user_id_unchecked_no_group_by: with_user_id_unchecked_no_group_by =
            with_user_id_unchecked_no_group_by(user_id);
        let album: AsSelect<Album, crate::orm::Type> = Album::as_select();
        with_user_id_unchecked_no_group_by.group_by(albums::id).select(album)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::{audio, picture};
    use crate::test::{Mock, mock};

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

        let album: audio::Album = Faker.fake();

        music_folder
            .add_audio_filesystem::<&str>()
            .album(album.clone())
            .picture(picture)
            .dir_picture(dir_picture)
            .depth(0)
            .n_song(10)
            .call()
            .await;

        let album = query::with_user_id_unchecked(mock.user_id(0).await)
            .get_result(&mut mock.get().await)
            .await
            .unwrap();
        if has_dir_picture {
            assert_eq!(album.cover_art, dir_picture_id);
        } else if has_picture {
            assert_eq!(album.cover_art, picture_id);
        } else {
            assert!(album.cover_art.is_none());
        }
    }
}
