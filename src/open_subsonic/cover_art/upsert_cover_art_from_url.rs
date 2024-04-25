use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use mime_guess::Mime;
use uuid::Uuid;

use super::upsert_cover_art_from_data;
use crate::{DatabasePool, OSError};

pub async fn upsert_cover_art_from_url<P: AsRef<Path>>(
    pool: &DatabasePool,
    art_dir: P,
    url: &str,
) -> Result<Uuid> {
    let response = reqwest::get(url).await?.error_for_status()?;
    let file_mime = Mime::from_str(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .ok_or_else(|| OSError::NotFound("Spotify cover art content-type".into()))?
            .to_str()?,
    )?;
    let file_format = file_mime.subtype().as_str();
    let file_data = response.bytes().await?;
    upsert_cover_art_from_data(pool, art_dir, file_data, file_format).await
}

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;

    use super::*;
    use crate::models::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_upsert_cover_art_from_url() {
        let infra = Infra::new().await;
        let artist_art_dir = infra.fs.art_config.artist_dir.as_ref().unwrap();

        let cover_art_id = upsert_cover_art_from_url(
            infra.pool(),
            artist_art_dir,
            "https://picsum.photos/400/500.jpg",
        )
        .await
        .unwrap();
        let cover_art_path = cover_arts::table
            .filter(cover_arts::id.eq(cover_art_id))
            .select(cover_arts::CoverArt::as_select())
            .get_result(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .to_path(artist_art_dir);

        let image = image::open(cover_art_path).unwrap();
        assert_eq!(image.height(), 500);
        assert_eq!(image.width(), 400);
    }
}
