use std::time::Duration;

use educe::Educe;
use s3::Client;
use time::OffsetDateTime;
use typed_path::Utf8TypedPath;

use super::{entry, path};
use crate::file::{self, audio};
use crate::http::binary;
use crate::{Error, config, error};
#[derive(Clone, Educe)]
#[educe(Debug)]
pub struct Filesystem {
    #[educe(Debug(ignore))]
    client: Client,
    presigned_duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct Path<'b, 'k> {
    pub bucket: &'b str,
    pub key: &'k str,
}

impl Filesystem {
    pub fn new(_tls: &config::filesystem::Tls, s3: &config::filesystem::S3) -> Self {
        let client = Client::builder(&s3.endpoint_url)
            .expect("Could not initialize s3 client builder")
            .region(&s3.region)
            .addressing_style(if s3.use_path_style_endpoint {
                s3::AddressingStyle::Path
            } else {
                s3::AddressingStyle::VirtualHosted
            })
            .max_attempts(s3.max_attempts)
            .timeout(Duration::from_secs(s3.timeout))
            .auth(
                s3::Auth::from_env()
                    .expect("Could not initialize aws authentication from environment variable"),
            )
            .build()
            .expect("Could not build s3 client");

        Self { client, presigned_duration: Duration::from_mins(s3.presigned_duration) }
    }

    pub fn split<'b, 'k, 'p: 'b + 'k>(
        path: impl Into<Utf8TypedPath<'p>>,
    ) -> Result<Path<'b, 'k>, Error> {
        let path = path.into();
        if let Utf8TypedPath::Unix(path) = path {
            if path.is_absolute()
                && let Some(path) = path.as_str().strip_prefix('/')
            {
                if let Some((bucket, key)) = path.split_once('/') {
                    Ok(Path { bucket, key })
                } else {
                    Ok(Path { bucket: path, key: "" })
                }
            } else {
                error::Kind::InvalidAbsolutePath(path.to_typed_path_buf()).into()
            }
        } else {
            error::Kind::InvalidTypedPathPlatform(path.to_path_buf()).into()
        }
    }

    #[cfg(test)]
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl super::Trait for Filesystem {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        let Path { bucket, key } = Self::split(path)?;
        self.client.objects().list_v2(bucket).prefix(key).max_keys(1).send().await?;
        Ok(())
    }

    async fn scan_folder(
        &self,
        sender: entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        let Path { bucket, key } = Self::split(prefix)?;
        let prefix = key;
        let mut pager = self.client.objects().list_v2(bucket).prefix(prefix).pager();
        let bucket = path::S3::from_str("/").join(bucket);

        while let Some(output) = pager.next_page().await? {
            for content in output.contents {
                sender.send(bucket.join(&content.key), &content).await?;
            }
        }

        Ok(())
    }

    async fn exists(&self, path: Utf8TypedPath<'_>) -> Result<bool, Error> {
        let Path { bucket, key } = Self::split(path)?;
        let result = self.client.objects().head(bucket, key).send().await;
        if let Err(error) = result {
            let error: Error = error.into();
            if error.status_code == axum::http::StatusCode::NOT_FOUND {
                Ok(false)
            } else {
                Err(error)
            }
        } else {
            Ok(true)
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        let Path { bucket, key } = Self::split(path)?;
        Ok(self.client.objects().get(bucket, key).send().await?.bytes().await?.into())
    }

    async fn read_to_string(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        String::from_utf8(self.read(path).await?).map_err(Error::from)
    }

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        let path = source.path.to_path();
        let Path { bucket, key } = Self::split(path)?;
        let stream = self
            .client
            .objects()
            .get(bucket, key)
            .range_bytes(offset.unwrap_or(0), source.property.size.get().into())
            .send()
            .await
            .map_err(Error::from)?
            .body;
        binary::Response::from_body(
            axum::body::Body::from_stream(stream),
            &source.property,
            offset,
            #[cfg(test)]
            None,
        )
    }

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        let Path { bucket, key } = Self::split(path)?;
        Ok(self
            .client
            .objects()
            .presign_get(bucket, key)
            .expires_in(self.presigned_duration)
            .build()?
            .url
            .into())
    }
}

impl entry::Metadata for s3::types::Object {
    fn size(&self) -> Result<usize, Error> {
        Ok(self.size.try_into()?)
    }

    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error> {
        Ok(self
            .last_modified
            .as_deref()
            .map(|timestamp| {
                OffsetDateTime::parse(
                    timestamp,
                    &time::format_description::well_known::Iso8601::DEFAULT,
                )
            })
            .transpose()?)
    }
}
