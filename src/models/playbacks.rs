use diesel::prelude::*;
pub use playbacks::*;
use time::OffsetDateTime;
use uuid::Uuid;

pub use crate::schema::playbacks;

#[derive(Insertable)]
#[diesel(table_name = playbacks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewScrobble {
    pub user_id: Uuid,
    pub song_id: Uuid,
    pub updated_at: Option<OffsetDateTime>,
}
