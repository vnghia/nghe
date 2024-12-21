use diesel::prelude::*;
use nghe_api::id3;
use o2o::o2o;
use uuid::Uuid;

use crate::orm::artists;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_into(id3::artist::Required)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}

pub mod query {
    use diesel::dsl::auto_type;

    use super::*;
    use crate::orm::{songs_album_artists, songs_artists};

    #[auto_type]
    pub fn album() -> _ {
        artists::table.on(artists::id.eq(songs_album_artists::album_artist_id))
    }

    #[auto_type]
    pub fn song() -> _ {
        artists::table.on(artists::id.eq(songs_artists::artist_id))
    }
}
