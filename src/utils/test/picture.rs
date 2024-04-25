use std::io::Cursor;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use image::{ImageFormat, Rgb, RgbImage};
use lofty::picture::Picture;
use uuid::Uuid;

use crate::config::ArtConfig;
use crate::models::*;
use crate::DatabasePool;

pub fn fake(force_some: bool) -> Option<Picture> {
    if Faker.fake() || force_some {
        let width = (100..=200).fake();
        let height = (100..=200).fake();
        let mut cursor = Cursor::new(vec![]);
        RgbImage::from_fn(width, height, |_, _| {
            Rgb::from([Faker.fake(), Faker.fake(), Faker.fake()])
        })
        .write_to(&mut cursor, if Faker.fake() { ImageFormat::Jpeg } else { ImageFormat::Png })
        .unwrap();
        cursor.set_position(0);
        Some(Picture::from_reader(&mut cursor).unwrap())
    } else {
        None
    }
}

pub async fn from_id(
    pool: &DatabasePool,
    cover_art_id: Option<Uuid>,
    art_config: &ArtConfig,
) -> Option<Picture> {
    if let Some(cover_art_id) = cover_art_id
        && let Some(art_path) = art_config.song_path.as_ref()
    {
        let song_cover_art = songs::table
            .inner_join(cover_arts::table)
            .filter(cover_arts::id.eq(cover_art_id))
            .select(cover_arts::CoverArt::as_select())
            .get_result(&mut pool.get().await.unwrap())
            .await
            .unwrap();
        let mut art_file = std::fs::File::open(song_cover_art.to_path(art_path)).unwrap();
        Some(Picture::from_reader(&mut art_file).unwrap())
    } else {
        None
    }
}
