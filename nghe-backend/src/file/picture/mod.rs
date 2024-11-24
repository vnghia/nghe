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

#[derive(Debug, o2o)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
#[ref_into(cover_arts::Upsert<'s>)]
pub struct Picture<'s, 'd> {
    #[into(~.as_ref().map(std::convert::AsRef::as_ref).map(Cow::Borrowed))]
    pub source: Option<Cow<'s, str>>,
    #[into(~.into())]
    pub property: Property<Format>,
    #[ghost]
    pub data: Cow<'d, [u8]>,
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

impl<'d> TryFrom<&'d LoftyPicture> for Picture<'static, 'd> {
    type Error = Error;

    fn try_from(value: &'d LoftyPicture) -> Result<Self, Self::Error> {
        Picture::new(
            None,
            value.mime_type().ok_or_else(|| Error::MediaPictureMissingFormat)?.try_into()?,
            value.data(),
        )
    }
}

impl<'s, 'd> Picture<'s, 'd> {
    pub const FILENAME: &'static str = "cover_art";
    pub const TEST_DESCRIPTION: &'static str = "nghe-picture-test-description";

    pub fn new(
        source: Option<Cow<'s, str>>,
        format: Format,
        data: impl Into<Cow<'d, [u8]>>,
    ) -> Result<Self, Error> {
        let data = data.into();
        let property = Property::new(format, &data)?;
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

    use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use fake::{Dummy, Fake, Faker};
    use image::{ImageFormat, Rgb, RgbImage};
    use lofty::picture::PictureType;

    use super::*;
    use crate::file;
    use crate::schema::songs;
    use crate::test::Mock;

    impl Dummy<Faker> for Picture<'_, '_> {
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

    impl<'s> Picture<'s, '_> {
        async fn load(dir: impl AsRef<Utf8NativePath>, upsert: cover_arts::Upsert<'s>) -> Self {
            let property: file::Property<Format> = upsert.property.into();
            let path = property.path(dir, Self::FILENAME);
            let data = tokio::fs::read(path).await.unwrap();
            Self { source: upsert.source, property, data: data.into() }
        }

        pub async fn query_song(mock: &Mock, song_id: Uuid) -> Option<Self> {
            if let Some(ref dir) = mock.config.cover_art.dir {
                let upsert = cover_arts::table
                    .inner_join(songs::table)
                    .filter(songs::id.eq(song_id))
                    .select(cover_arts::Upsert::as_select())
                    .get_result(&mut mock.get().await)
                    .await
                    .optional()
                    .unwrap();
                if let Some(upsert) = upsert { Some(Self::load(dir, upsert).await) } else { None }
            } else {
                None
            }
        }
    }
}
