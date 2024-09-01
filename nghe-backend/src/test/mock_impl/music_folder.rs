use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::orm::music_folders;

pub struct Mock<'a> {
    mock: &'a super::Mock,
    pub music_folder: music_folders::MusicFolder<'static>,
}

impl<'a> Mock<'a> {
    pub async fn new(mock: &'a super::Mock, index: usize) -> Self {
        Self {
            mock,
            music_folder: music_folders::table
                .select(music_folders::MusicFolder::as_select())
                .order_by(music_folders::schema::created_at)
                .offset(index.try_into().unwrap())
                .first(&mut mock.get().await)
                .await
                .unwrap(),
        }
    }
}
