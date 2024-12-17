use std::borrow::Cow;

use diesel::sql_types::Text;
use diesel::{AsExpression, ExpressionMethods, FromSqlRow, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use educe::Educe;
use lofty::picture::{MimeType, Picture as LoftyPicture};
use nghe_api::common::format;
use o2o::o2o;
use strum::{EnumString, IntoStaticStr};
use typed_path::{Utf8PlatformPath, Utf8TypedPath, Utf8TypedPathBuf};
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
#[cfg_attr(test, derive(fake::Dummy, o2o, strum::EnumIter))]
#[cfg_attr(test, owned_into(MimeType))]
pub enum Format {
    Png,
    Jpeg,
}

#[derive(o2o, Educe)]
#[educe(Debug)]
#[ref_into(cover_arts::Upsert<'s>)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
pub struct Picture<'s, 'd> {
    #[into(~.as_ref().map(|value| value.as_str().into()))]
    pub source: Option<Cow<'s, str>>,
    #[into(~.into())]
    pub property: Property<Format>,
    #[ghost]
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

impl<'d> TryFrom<&'d LoftyPicture> for Picture<'static, 'd> {
    type Error = Error;

    fn try_from(value: &'d LoftyPicture) -> Result<Self, Self::Error> {
        Picture::new(
            None,
            value.mime_type().ok_or_else(|| error::Kind::MissingPictureFormat)?.try_into()?,
            value.data(),
        )
    }
}

impl<'s, 'd> Picture<'s, 'd> {
    pub const FILENAME: &'static str = "cover_art";
    pub const TEST_DESCRIPTION: &'static str = "nghe-picture-test-description";

    fn new(
        source: Option<Cow<'s, str>>,
        format: Format,
        data: impl Into<Cow<'d, [u8]>>,
    ) -> Result<Self, Error> {
        let data = data.into();
        let property = Property::new(format, &data)?;
        Ok(Self { source, property, data })
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
    ) -> Result<Uuid, Error> {
        self.dump(dir).await?;
        let upsert: cover_arts::Upsert = self.into();
        upsert.insert(database).await
    }

    pub async fn query_source(
        database: &Database,
        path: impl AsRef<str>,
    ) -> Result<Option<Uuid>, Error> {
        cover_arts::table
            .filter(cover_arts::source.eq(path.as_ref()))
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
                if !full && let Some(picture_id) = Self::query_source(database, &path).await? {
                    return Ok(Some(picture_id));
                } else if let Some(picture) = Picture::load(filesystem, path).await? {
                    return Ok(Some(picture.upsert(database, art_dir).await?));
                }
            }
        }
        Ok(None)
    }
}

impl Picture<'static, 'static> {
    pub async fn load(
        filesystem: &filesystem::Impl<'_>,
        source: Utf8TypedPathBuf,
    ) -> Result<Option<Self>, Error> {
        let path = source.to_path();
        if filesystem.exists(path).await? {
            let format = {
                let format = path
                    .extension()
                    .ok_or_else(|| error::Kind::MissingPathExtension(path.to_path_buf()))?;
                format
                    .parse()
                    .map_err(|_| error::Kind::UnsupportedPictureFormat(format.to_owned()))?
            };
            let data = filesystem.read(path).await?;
            return Ok(Some(Picture::new(Some(source.into_string().into()), format, data)?));
        }
        Ok(None)
    }
}

impl<'s> Picture<'s, 'static> {
    pub async fn fetch(
        client: &reqwest::Client,
        source: impl Into<Cow<'s, str>>,
    ) -> Result<Self, Error> {
        let source = source.into();
        let response = client.get(source.as_str()).send().await?.error_for_status()?;
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
        Picture::new(Some(source), format, data.to_vec())
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
    use image::{ImageFormat, Rgb, RgbImage};
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

    impl Dummy<Faker> for Picture<'_, '_> {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            let format: Format = config.fake_with_rng(rng);

            let mut cursor = Cursor::new(vec![]);
            RgbImage::from_fn(
                (100..=200).fake_with_rng(rng),
                (100..=200).fake_with_rng(rng),
                |_, _| Rgb::from(Faker.fake_with_rng::<[u8; 3], _>(rng)),
            )
            .write_to(&mut cursor, match format {
                Format::Png => ImageFormat::Png,
                Format::Jpeg => ImageFormat::Jpeg,
            })
            .unwrap();
            cursor.set_position(0);

            Self::new(None, format, cursor.into_inner()).unwrap()
        }
    }

    impl From<Picture<'_, '_>> for LoftyPicture {
        fn from(value: Picture<'_, '_>) -> Self {
            Self::new_unchecked(
                PictureType::Other,
                Some(value.property.format.into()),
                Some(Picture::TEST_DESCRIPTION.to_owned()),
                value.data.into_owned(),
            )
        }
    }

    impl Picture<'_, '_> {
        pub async fn upsert_mock(&self, mock: &Mock) -> Uuid {
            self.upsert(mock.database(), mock.config.cover_art.dir.as_ref().unwrap()).await.unwrap()
        }

        pub async fn dump_filesystem(&self, filesystem: &filesystem::Impl<'_>) {
            filesystem
                .write(filesystem.path().from_str(self.source.as_ref().unwrap()), &self.data)
                .await;
        }
    }

    impl<'s> Picture<'s, '_> {
        async fn load_cache(
            dir: impl AsRef<Utf8PlatformPath>,
            upsert: cover_arts::Upsert<'s>,
        ) -> Self {
            let property: file::Property<Format> = upsert.property.try_into().unwrap();
            let path = property.path(dir, Self::FILENAME);
            let data = tokio::fs::read(path).await.unwrap();
            Self { source: upsert.source, property, data: data.into() }
        }

        pub async fn query_song(mock: &Mock, id: Uuid) -> Option<Self> {
            if let Some(ref dir) = mock.config.cover_art.dir {
                let upsert = cover_arts::table
                    .inner_join(songs::table)
                    .filter(songs::id.eq(id))
                    .select(cover_arts::Upsert::as_select())
                    .get_result(&mut mock.get().await)
                    .await
                    .optional()
                    .unwrap();
                if let Some(upsert) = upsert {
                    Some(Self::load_cache(dir, upsert).await)
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub async fn query_album(mock: &Mock, id: Uuid) -> Option<Self> {
            if let Some(ref dir) = mock.config.cover_art.dir {
                let upsert = cover_arts::table
                    .inner_join(albums::table)
                    .filter(albums::id.eq(id))
                    .select(cover_arts::Upsert::as_select())
                    .get_result(&mut mock.get().await)
                    .await
                    .optional()
                    .unwrap();
                if let Some(upsert) = upsert {
                    Some(Self::load_cache(dir, upsert).await)
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub fn with_source(self, source: Option<impl Into<Cow<'s, str>>>) -> Self {
            Self { source: source.map(std::convert::Into::into), ..self }
        }
    }

    impl Picture<'static, 'static> {
        pub async fn scan_filesystem(
            filesystem: &filesystem::Impl<'_>,
            config: &config::CoverArt,
            dir: Utf8TypedPath<'_>,
        ) -> Option<Self> {
            for name in &config.names {
                let path = dir.join(name);
                if let Some(picture) = Self::load(&filesystem.main(), path).await.unwrap() {
                    return Some(picture);
                }
            }
            None
        }

        pub async fn fake_filesystem(filesystem: &filesystem::Impl<'_>, format: Format) -> Self {
            let path = filesystem.prefix().join(format.name());
            let picture = Picture {
                source: Some(path.to_string().into()),
                property: Property { format, ..Faker.fake() },
                data: fake::vec![u8; 100..200].into(),
            };
            filesystem.write(path.to_path(), &picture.data).await;
            picture
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

        let picture = Picture::fake_filesystem(&filesystem, format).await;
        let picture_id = picture.upsert_mock(&mock).await;
        Picture::fake_filesystem(&filesystem, format).await;

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
