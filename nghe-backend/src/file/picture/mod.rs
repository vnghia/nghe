use std::borrow::Cow;

use diesel::sql_types::Text;
use diesel::{
    AsExpression, ExpressionMethods, FromSqlRow, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use educe::Educe;
use lofty::picture::{MimeType, Picture as LoftyPicture};
use nghe_api::common::format;
use strum::{EnumString, IntoStaticStr};
use typed_path::{Utf8PlatformPath, Utf8PlatformPathBuf, Utf8TypedPath};
use uuid::Uuid;

use super::Property;
use crate::database::Database;
use crate::filesystem::Trait as _;
use crate::orm::cover_arts;
use crate::orm::upsert::Insert;
use crate::{Error, config, error, filesystem};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    IntoStaticStr,
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(test, derive(fake::Dummy, o2o::o2o, strum::EnumIter))]
#[cfg_attr(test, owned_into(MimeType))]
#[cfg_attr(test, owned_into(image::ImageFormat))]
pub enum Format {
    Png,
    #[strum(serialize = "jpeg", serialize = "jpg")]
    Jpeg,
}

#[derive(Educe)]
#[educe(Debug)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
pub struct Picture<'d> {
    pub property: Property<Format>,
    #[educe(Debug(ignore))]
    pub data: Cow<'d, [u8]>,
}

impl TryFrom<&MimeType> for Format {
    type Error = Error;

    fn try_from(value: &MimeType) -> Result<Self, Self::Error> {
        match value {
            MimeType::Png => Ok(Self::Png),
            MimeType::Jpeg => Ok(Self::Jpeg),
            _ => error::Kind::UnsupportedPictureFormat(value.as_str().to_owned()).into(),
        }
    }
}

impl format::Trait for Format {
    const CACHE_CONTROL: format::CacheControl =
        format::CacheControl { duration: std::time::Duration::from_days(365), immutable: true };

    fn mime(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }

    fn extension(&self) -> &'static str {
        self.into()
    }
}

impl super::Property<Format> {
    pub async fn query_cover_art(database: &Database, id: Uuid) -> Result<Self, Error> {
        cover_arts::table
            .filter(cover_arts::id.eq(id))
            .select(cover_arts::Property::as_select())
            .get_result(&mut database.get().await?)
            .await?
            .try_into()
    }

    pub fn picture_path(&self, base: impl AsRef<Utf8PlatformPath>) -> Utf8PlatformPathBuf {
        self.path(base, Picture::FILENAME)
    }
}

impl<'d> TryFrom<&'d LoftyPicture> for Picture<'d> {
    type Error = Error;

    fn try_from(value: &'d LoftyPicture) -> Result<Self, Self::Error> {
        Picture::new(
            value.mime_type().ok_or_else(|| error::Kind::MissingPictureFormat)?.try_into()?,
            value.data(),
        )
    }
}

impl<'d> Picture<'d> {
    pub const FILENAME: &'static str = "cover_art";
    pub const TEST_DESCRIPTION: &'static str = "nghe-picture-test-description";

    fn new(format: Format, data: impl Into<Cow<'d, [u8]>>) -> Result<Self, Error> {
        let data = data.into();
        let property = Property::new(format, &data)?;
        Ok(Self { property, data })
    }

    pub async fn dump(&self, dir: impl AsRef<Utf8PlatformPath>) -> Result<(), Error> {
        let path = self.property.path_create_dir(dir, Self::FILENAME).await?;
        // Path already contains information about hash, size and format so we don't need to worry
        // about stale data.
        if !tokio::fs::try_exists(&path).await? {
            tokio::fs::write(path, &self.data).await?;
        }
        Ok(())
    }

    pub async fn upsert(
        &self,
        database: &Database,
        dir: impl AsRef<Utf8PlatformPath>,
        source: Option<impl AsRef<str>>,
    ) -> Result<Uuid, Error> {
        self.dump(dir).await?;
        let upsert = cover_arts::Upsert {
            source: source.as_ref().map(AsRef::as_ref).map(Cow::Borrowed),
            property: self.property.into(),
        };
        upsert.insert(database).await
    }

    pub async fn query_source(
        database: &Database,
        source: impl AsRef<str>,
    ) -> Result<Option<Uuid>, Error> {
        cover_arts::table
            .filter(cover_arts::source.eq(source.as_ref()))
            .select(cover_arts::id)
            .get_result(&mut database.get().await?)
            .await
            .optional()
            .map_err(Error::from)
    }

    pub async fn scan(
        database: &Database,
        filesystem: &filesystem::Impl<'_>,
        config: &config::CoverArt,
        full: bool,
        dir: Utf8TypedPath<'_>,
    ) -> Result<Option<Uuid>, Error> {
        if let Some(ref art_dir) = config.dir {
            for name in &config.names {
                let path = dir.join(name);
                let path = path.to_path();
                if !full && let Some(picture_id) = Self::query_source(database, path).await? {
                    return Ok(Some(picture_id));
                } else if let Some(picture) = Picture::load(filesystem, path).await? {
                    return Ok(Some(picture.upsert(database, art_dir, Some(path)).await?));
                }
            }
        }
        Ok(None)
    }
}

impl Picture<'static> {
    pub async fn load(
        filesystem: &filesystem::Impl<'_>,
        path: Utf8TypedPath<'_>,
    ) -> Result<Option<Self>, Error> {
        if filesystem.exists(path).await? {
            let format = {
                let format = path
                    .extension()
                    .ok_or_else(|| error::Kind::MissingPathExtension(path.to_path_buf()))?;
                format
                    .to_lowercase()
                    .parse()
                    .map_err(|_| error::Kind::UnsupportedPictureFormat(format.to_owned()))?
            };
            let data = filesystem.read(path).await?;
            return Ok(Some(Picture::new(format, data)?));
        }
        Ok(None)
    }

    pub async fn fetch(client: &reqwest::Client, url: impl AsRef<str>) -> Result<Self, Error> {
        let response = client.get(url.as_ref()).send().await?.error_for_status()?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .ok_or_else(|| error::Kind::MissingPictureFormat)?
            .to_str()?;
        let format = content_type
            .split_once('/')
            .and_then(|(ty, subtype)| if ty == "image" { subtype.parse().ok() } else { None })
            .ok_or_else(|| error::Kind::UnsupportedPictureFormat(content_type.to_owned()))?;
        let data = response.bytes().await?;
        Picture::new(format, data.to_vec())
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use std::io::Cursor;

    use concat_string::concat_string;
    use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use fake::{Dummy, Fake, Faker};
    use image::{Rgb, RgbImage};
    use lofty::picture::PictureType;

    use super::*;
    use crate::file;
    use crate::orm::albums;
    use crate::schema::songs;
    use crate::test::filesystem::Trait;
    use crate::test::{Mock, filesystem};

    impl Format {
        pub fn name(self) -> String {
            concat_string!("cover", ".", std::convert::Into::<&'static str>::into(self))
        }
    }

    impl Dummy<Faker> for Picture<'_> {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            let format: Format = config.fake_with_rng(rng);

            let mut data = Vec::new();
            RgbImage::from_fn(
                (100..=200).fake_with_rng(rng),
                (100..=200).fake_with_rng(rng),
                |_, _| Rgb::from(Faker.fake_with_rng::<[u8; 3], _>(rng)),
            )
            .write_to(&mut Cursor::new(&mut data), format.into())
            .unwrap();

            Self::new(format, data).unwrap()
        }
    }

    impl From<Picture<'_>> for LoftyPicture {
        fn from(value: Picture<'_>) -> Self {
            Self::new_unchecked(
                PictureType::Other,
                Some(value.property.format.into()),
                Some(Picture::TEST_DESCRIPTION.to_owned()),
                value.data.into_owned(),
            )
        }
    }

    impl Picture<'_> {
        pub async fn upsert_mock(&self, mock: &Mock, source: Option<impl AsRef<str>>) -> Uuid {
            self.upsert(mock.database(), mock.config.cover_art.dir.as_ref().unwrap(), source)
                .await
                .unwrap()
        }

        async fn load_cache(
            dir: impl AsRef<Utf8PlatformPath>,
            property: cover_arts::Property,
        ) -> Self {
            let property: file::Property<Format> = property.try_into().unwrap();
            let path = property.path(dir, Self::FILENAME);
            let data = tokio::fs::read(path).await.unwrap();
            Self { property, data: data.into() }
        }

        pub async fn query_song(mock: &Mock, id: Uuid) -> Option<Self> {
            if let Some(ref dir) = mock.config.cover_art.dir {
                let property = cover_arts::table
                    .inner_join(songs::table)
                    .filter(songs::id.eq(id))
                    .select(cover_arts::Property::as_select())
                    .get_result(&mut mock.get().await)
                    .await
                    .optional()
                    .unwrap();
                if let Some(property) = property {
                    Some(Self::load_cache(dir, property).await)
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub async fn query_album(mock: &Mock, id: Uuid) -> Option<Self> {
            if let Some(ref dir) = mock.config.cover_art.dir {
                let property = cover_arts::table
                    .inner_join(albums::table)
                    .filter(albums::id.eq(id))
                    .select(cover_arts::Property::as_select())
                    .get_result(&mut mock.get().await)
                    .await
                    .optional()
                    .unwrap();
                if let Some(property) = property {
                    Some(Self::load_cache(dir, property).await)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    impl Picture<'static> {
        pub async fn scan_filesystem(
            filesystem: &filesystem::Impl<'_>,
            config: &config::CoverArt,
            dir: Utf8TypedPath<'_>,
        ) -> Option<Self> {
            for name in &config.names {
                if let Some(picture) =
                    Self::load(&filesystem.main(), dir.join(name).to_path()).await.unwrap()
                {
                    return Some(picture);
                }
            }
            None
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::filesystem::Trait;
    use crate::test::{Mock, mock};

    #[rstest]
    #[case("png", Format::Png)]
    #[case("jpeg", Format::Jpeg)]
    #[case("jpg", Format::Jpeg)]
    fn test_format(#[case] extension: &str, #[case] format: Format) {
        assert_eq!(extension.parse::<Format>().unwrap(), format);
    }

    #[rstest]
    #[tokio::test]
    async fn test_scan_full(
        #[future(awt)]
        #[with(0, 1)]
        mock: Mock,
        #[values(true, false)] full: bool,
    ) {
        let music_folder = mock.music_folder(0).await;
        let filesystem = music_folder.to_impl();
        let format: Format = Faker.fake();
        let path = filesystem.prefix().join(format.name());
        let path = path.to_path();

        let picture = Picture { data: fake::vec![u8; 100].into(), ..Faker.fake() };
        let picture_id = picture.upsert_mock(&mock, Some(&path)).await;
        filesystem.write(path, &picture.data).await;
        filesystem.write(path, &fake::vec![u8; 100]).await;

        let scanned_picture_id = Picture::scan(
            mock.database(),
            &filesystem.main(),
            &mock.config.cover_art,
            full,
            filesystem.prefix(),
        )
        .await
        .unwrap()
        .unwrap();
        // Full mode will take a newly created picture from the filesystem so we will have a
        // different id than the current one.
        assert_eq!(scanned_picture_id != picture_id, full);
    }
}
