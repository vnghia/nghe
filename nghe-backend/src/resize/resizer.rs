use std::io::{Cursor, Write};

use atomic_write_file::AtomicWriteFile;
use educe::Educe;
use image::ImageReader;
use tokio::sync::oneshot::{Receiver, Sender};
use typed_path::Utf8PlatformPathBuf;

use crate::Error;

#[derive(Educe)]
#[educe(Debug)]
pub struct Resizer {
    #[educe(Debug(ignore))]
    tx: Sender<Vec<u8>>,
    input: Utf8PlatformPathBuf,
    output: Option<AtomicWriteFile>,
    size: u32,
}

impl Resizer {
    pub fn spawn(
        input: Utf8PlatformPathBuf,
        output: Option<Utf8PlatformPathBuf>,
        size: u32,
    ) -> (Receiver<Vec<u8>>, tokio::task::JoinHandle<Result<(), Error>>) {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let span = tracing::Span::current();
        let handle = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();

            let output = output.map(AtomicWriteFile::open).transpose()?;
            let resizer = Self { tx, input, output, size };
            resizer.resize()
        });

        (rx, handle)
    }

    pub fn resize(mut self) -> Result<(), Error> {
        let image = ImageReader::open(&self.input)?.decode()?.resize(
            self.size,
            self.size,
            image::imageops::FilterType::Triangle,
        );

        let mut data: Vec<u8> = Vec::new();
        image.write_to(&mut Cursor::new(&mut data), image::ImageFormat::WebP)?;

        self.output.as_mut().map(|output| output.write_all(&data)).transpose()?;
        let _ = self.tx.send(data);

        self.output.map(AtomicWriteFile::commit).transpose()?;
        Ok(())
    }
}
