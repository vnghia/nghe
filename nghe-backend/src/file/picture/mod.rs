use std::borrow::Cow;

use lofty::picture::{MimeType, Picture as LoftyPicture};
use nghe_api::common::format;
use strum::{EnumString, IntoStaticStr};

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, IntoStaticStr)]
#[cfg_attr(test, derive(fake::Dummy, o2o::o2o))]
#[cfg_attr(test, owned_into(MimeType))]
pub enum Format {
    Png,
    Jpeg,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Picture<'a> {
    pub format: Format,
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
        Ok(Picture {
            format: value
                .mime_type()
                .ok_or_else(|| Error::MediaPictureMissingFormat)?
                .try_into()?,
            data: value.data().into(),
        })
    }
}

impl Picture<'_> {
    pub const TEST_DESCRIPTION: &'static str = "nghe-picture-test-description";
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use fake::{Dummy, Fake, Faker};
    use image::{ImageFormat, Rgb, RgbImage};
    use lofty::picture::PictureType;

    use super::*;

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

            Self { format, data: cursor.into_inner().into() }
        }
    }

    impl From<Picture<'_>> for LoftyPicture {
        fn from(value: Picture<'_>) -> Self {
            Self::new_unchecked(
                PictureType::Other,
                Some(value.format.into()),
                Some(Picture::TEST_DESCRIPTION.to_owned()),
                value.data.into_owned(),
            )
        }
    }
}
