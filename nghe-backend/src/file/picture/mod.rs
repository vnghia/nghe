use std::borrow::Cow;

use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use lofty::picture::{MimeType, Picture as LoftyPicture};
use nghe_api::common::format;
use o2o::o2o;
use strum::{EnumString, IntoStaticStr};
use typed_path::Utf8NativePath;
use uuid::Uuid;

use super::Property;
use crate::database::Database;
use crate::orm::cover_arts;
use crate::orm::upsert::Insert;
use crate::Error;

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
#[strum(serialize_all = "snake_case")]
#[cfg_attr(test, derive(fake::Dummy, o2o))]
#[cfg_attr(test, owned_into(MimeType))]
pub enum Format {
    Png,
    Jpeg,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(test, derive(fake::Dummy))]
pub enum Source {
    Embed,
}

#[derive(Debug, o2o)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
#[ref_into(cover_arts::Upsert)]
pub struct Picture<'a> {
    pub source: Source,
    #[map(~.into())]
    pub property: Property<Format>,
    #[ghost]
    pub data: Cow<'a, [u8]>,
}

impl TryFrom<&MimeType> for Format {
    type Error = Error;

    fn try_from(value: &MimeType) -> Result<Self, Self::Error> {
        match value {
            MimeType::Png => Ok(Self::Png),
            MimeType::Jpeg => Ok(Self::Jpeg),
            _ => Err(Self::Error::MediaPictureUnsupportedFormat(value.as_str().to_owned())),
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

impl<'a> TryFrom<&'a LoftyPicture> for Picture<'a> {
    type Error = Error;

    fn try_from(value: &'a LoftyPicture) -> Result<Self, Self::Error> {
        Picture::new(
            value.data(),
            Source::Embed,
            value.mime_type().ok_or_else(|| Error::MediaPictureMissingFormat)?.try_into()?,
        )
    }
}

impl<'a> Picture<'a> {
    pub const FILENAME: &'static str = "cover_art";
    pub const TEST_DESCRIPTION: &'static str = "nghe-picture-test-description";

    pub fn new(
        data: impl Into<Cow<'a, [u8]>>,
        source: Source,
        format: Format,
    ) -> Result<Self, Error> {
        let data = data.into();
        let property = Property::new(&data, format)?;
        Ok(Self { source, property, data })
    }

    pub async fn dump(&self, dir: impl AsRef<Utf8NativePath>) -> Result<(), Error> {
        let path = self.property.path_create_dir(dir, Self::FILENAME).await?;
        tokio::fs::write(path, &self.data).await?;
        Ok(())
    }

    pub async fn upsert(
        &self,
        database: &Database,
        dir: impl AsRef<Utf8NativePath>,
    ) -> Result<Uuid, Error> {
        // TODO: Checking for its existence before dump.
        self.dump(dir).await?;
        let upsert: cover_arts::Upsert = self.into();
        upsert.insert(database).await
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use fake::{Dummy, Fake, Faker};
    use image::{ImageFormat, Rgb, RgbImage};
    use lofty::picture::PictureType;

    use super::*;
    use crate::file;

    impl Dummy<Faker> for Picture<'_> {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            let format: Format = config.fake_with_rng(rng);

            let mut cursor = Cursor::new(vec![]);
            RgbImage::from_fn(
                (100..=200).fake_with_rng(rng),
                (100..=200).fake_with_rng(rng),
                |_, _| Rgb::from(Faker.fake_with_rng::<[u8; 3], _>(rng)),
            )
            .write_to(
                &mut cursor,
                match format {
                    Format::Png => ImageFormat::Png,
                    Format::Jpeg => ImageFormat::Jpeg,
                },
            )
            .unwrap();
            cursor.set_position(0);

            Self::new(cursor.into_inner(), Source::Embed, format).unwrap()
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
        pub async fn load(dir: impl AsRef<Utf8NativePath>, cover_art: cover_arts::Upsert) -> Self {
            let property: file::Property<Format> = cover_art.property.into();
            let path = property.path(dir, Self::FILENAME);
            let data = tokio::fs::read(path).await.unwrap();
            Self { source: cover_art.source, property, data: data.into() }
        }
    }
}
