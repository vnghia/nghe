use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_async::RunQueryDsl;
use nghe_api::id3;
use uuid::Uuid;

use super::{album, artist};
use crate::database::Database;
use crate::orm::albums;
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct ArtistWithAlbums {
    #[diesel(embed)]
    pub artist: artist::Artist,
    #[diesel(select_expression = sql(
        "array_remove(array_agg(distinct(albums.id)), null) album_ids"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub albums: Vec<Uuid>,
}

impl ArtistWithAlbums {
    pub async fn try_into_api(self, database: &Database) -> Result<id3::ArtistWithAlbum, Error> {
        Ok(id3::ArtistWithAlbum {
            artist: self.artist.try_into_api()?,
            album: album::query::unchecked()
                .filter(albums::id.eq_any(self.albums))
                .get_results(&mut database.get().await?)
                .await?
                .into_iter()
                .map(album::Album::try_into_api)
                .try_collect()?,
        })
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};
    use uuid::Uuid;

    use super::*;

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let artist: artist::query::with_user_id = artist::query::with_user_id(user_id);
        let artist_with_albums: AsSelect<ArtistWithAlbums, crate::orm::Type> =
            ArtistWithAlbums::as_select();
        artist.select(artist_with_albums)
    }
}
