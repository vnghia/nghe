use anyhow::Result;
use concat_string::concat_string;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use crate::models::*;
use crate::utils::fs::path::hash_size_to_path;
use crate::utils::fs::LocalPath;
use crate::DatabasePool;

pub async fn upsert_cover_art_from_data<P: AsRef<LocalPath>, D: AsRef<[u8]>>(
    pool: &DatabasePool,
    art_dir: P,
    file_data: D,
    file_format: &str,
) -> Result<Uuid> {
    let file_data = file_data.as_ref();
    let file_name = concat_string!("cover.", &file_format);
    let file_hash = xxh3_64(file_data);
    let file_size = file_data.len() as _;

    let art_dir = hash_size_to_path(art_dir, file_hash, file_size);
    tokio::fs::create_dir_all(&art_dir).await?;
    tokio::fs::write(art_dir.join(file_name), file_data).await?;

    diesel::insert_into(cover_arts::table)
        .values(cover_arts::NewCoverArt {
            format: file_format.into(),
            file_hash: file_hash as _,
            file_size: file_size as _,
        })
        .on_conflict((cover_arts::format, cover_arts::file_hash, cover_arts::file_size))
        .do_update()
        .set(cover_arts::upserted_at.eq(time::OffsetDateTime::now_utc()))
        .returning(cover_arts::id)
        .get_result::<Uuid>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}
