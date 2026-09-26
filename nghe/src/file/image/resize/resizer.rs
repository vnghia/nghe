use std::io::{Cursor, Write};

use ::image::ImageReader;
use atomic_write_file::AtomicWriteFile;
use typed_path::Utf8PlatformPathBuf;

use crate::Error;
use crate::file::image;

pub struct Resizer {
    input: Utf8PlatformPathBuf,
    output: Option<AtomicWriteFile>,
    size: u32,
    format: image::Format,
}

impl Resizer {
    pub async fn spawn(
        input: Utf8PlatformPathBuf,
        output: Option<Utf8PlatformPathBuf>,
        format: image::Format,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        let span = tracing::Span::current();
        tokio::task::spawn_blocking(move || {
            let _entered = span.enter();

            let output = output.map(AtomicWriteFile::open).transpose()?;
            Self { input, output, size, format }.resize()
        })
        .await?
    }

    pub fn resize(self) -> Result<Vec<u8>, Error> {
        let image = ImageReader::open(&self.input)?.decode()?.resize(
            self.size,
            self.size,
            ::image::imageops::FilterType::Triangle,
        );

        let mut data: Vec<u8> = Vec::new();
        image.write_to(&mut Cursor::new(&mut data), self.format.into())?;

        if let Some(mut output) = self.output {
            output.write_all(&data)?;
            output.commit()?;
        }
        Ok(data)
    }
}
