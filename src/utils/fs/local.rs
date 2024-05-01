use std::fmt::Debug;
use std::fs::Metadata;

use anyhow::Result;
use flume::Sender;
use ignore::types::TypesBuilder;
use ignore::{DirEntry, Error, WalkBuilder};
use tokio::task::JoinHandle;
use tracing::instrument;
use typed_path::{Utf8NativeEncoding, Utf8Path, Utf8PathBuf};

use super::FsTrait;
use crate::utils::path::{PathInfo, PathMetadata};
use crate::utils::song::file_type::{to_extension, FILETYPE_GLOB_PATTERN};

#[derive(Debug, Clone)]
pub struct LocalFs {
    pub scan_parallel: bool,
}

pub type LocalPath = Utf8Path<Utf8NativeEncoding>;
pub type LocalPathBuf = Utf8PathBuf<Utf8NativeEncoding>;

impl LocalFs {
    fn process_dir_entry(
        tx: &Sender<PathInfo<Self>>,
        entry: Result<DirEntry, Error>,
    ) -> ignore::WalkState {
        match try {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let path = entry.path();
            if metadata.is_file()
                && let Err(e) = tx.send(PathInfo::new(
                    path.to_str().expect("non utf-8 path encountered"),
                    &metadata,
                ))
            {
                tracing::error!(sending_walkdir_result = ?e);
                ignore::WalkState::Quit
            } else {
                ignore::WalkState::Continue
            }
        } {
            Ok(r) => r,
            Err::<_, anyhow::Error>(e) => {
                tracing::error!(walking_media_directory = ?e);
                ignore::WalkState::Continue
            }
        }
    }

    #[instrument(skip(tx))]
    fn walkdir<P: AsRef<Utf8Path<<Self as FsTrait>::E>> + Debug + Send + Sync>(
        prefix: P,
        tx: Sender<PathInfo<Self>>,
        scan_parallel: bool,
    ) {
        tracing::info!("start walking dir");

        let types = match try {
            let mut types = TypesBuilder::new();
            for (pattern, file_type) in &FILETYPE_GLOB_PATTERN {
                types.add(to_extension(file_type), pattern)?;
            }
            types.select("all").build()?
        } {
            Ok(r) => r,
            Err::<_, anyhow::Error>(e) => {
                tracing::error!(building_scan_pattern = ?e);
                return;
            }
        };

        let prefix = prefix.as_ref().as_str();
        if scan_parallel {
            WalkBuilder::new(prefix).types(types).build_parallel().run(|| {
                let span = tracing::Span::current();
                let tx = tx.clone();
                Box::new(move |entry| {
                    let _enter = span.enter();
                    Self::process_dir_entry(&tx, entry)
                })
            });
        } else {
            for entry in WalkBuilder::new(prefix).types(types).build() {
                Self::process_dir_entry(&tx, entry);
            }
        }

        tracing::info!("finish walking dir");
    }
}

impl From<&Metadata> for PathMetadata {
    fn from(value: &Metadata) -> Self {
        Self { size: value.len() as _ }
    }
}

#[async_trait::async_trait]
impl FsTrait for LocalFs {
    type E = Utf8NativeEncoding;

    async fn read<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(&self, path: P) -> Result<Vec<u8>> {
        tokio::fs::read(path.as_ref().as_str()).await.map_err(anyhow::Error::from)
    }

    async fn read_to_string<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<String> {
        tokio::fs::read_to_string(path.as_ref().as_str()).await.map_err(anyhow::Error::from)
    }

    fn scan_songs<P: AsRef<Utf8Path<Self::E>> + Debug + Send + Sync + 'static>(
        &self,
        prefix: P,
        tx: Sender<PathInfo<Self>>,
    ) -> JoinHandle<()> {
        let span = tracing::Span::current();
        let scan_parallel = self.scan_parallel;
        tokio::task::spawn_blocking(move || {
            let _enter = span.enter();
            Self::walkdir(prefix, tx, scan_parallel)
        })
    }
}

#[cfg(test)]
mod tests {
    use futures::{stream, StreamExt};
    use itertools::Itertools;

    use super::*;
    use crate::utils::song::file_type::SUPPORTED_EXTENSIONS;
    use crate::utils::test::Infra;

    const FS_INDEX: usize = 0;

    async fn wrap_scan_media_file(infra: &Infra, scan_parallel: bool) -> Vec<PathInfo<LocalFs>> {
        let (tx, rx) = flume::bounded(100);
        let scan_task =
            LocalFs { scan_parallel }.scan_songs(infra.fs.prefix(FS_INDEX).to_string(), tx);
        let mut result = vec![];
        while let Ok(r) = rx.recv_async().await {
            result.push(r);
        }
        scan_task.await.unwrap();
        result
    }

    #[tokio::test]
    async fn test_scan_media_files_no_filter() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(infra.fs.mkrelpaths(
            FS_INDEX,
            50,
            3,
            &SUPPORTED_EXTENSIONS.keys().collect_vec(),
        ))
        .then(move |path| async move { fs.mkfile(FS_INDEX, &path).await })
        .collect::<Vec<_>>()
        .await;

        let scanned_results = wrap_scan_media_file(&infra, false).await;
        let scanned_paths =
            scanned_results.iter().cloned().map(|result| result.path.to_string()).collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_extension() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(
            infra.fs.mkrelpaths(
                FS_INDEX,
                50,
                3,
                &[SUPPORTED_EXTENSIONS.keys().copied().collect_vec().as_slice(), &["txt", "rs"]]
                    .concat(),
            ),
        )
        .filter_map(move |path| async move {
            let path = fs.mkfile(FS_INDEX, &path).await;
            let extension = fs.extension(FS_INDEX, &path);
            if SUPPORTED_EXTENSIONS.contains_key(extension) { Some(path) } else { None }
        })
        .collect::<Vec<_>>()
        .await;

        let scanned_paths = wrap_scan_media_file(&infra, false)
            .await
            .into_iter()
            .map(|result| result.path.to_string())
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_dir() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(fs.mkrelpaths(
            FS_INDEX,
            50,
            3,
            &SUPPORTED_EXTENSIONS.keys().collect_vec(),
        ))
        .filter_map(move |path| async move {
            if rand::random::<bool>() {
                Some(fs.mkfile(FS_INDEX, &path).await)
            } else {
                fs.mkdir(FS_INDEX, &path).await;
                None
            }
        })
        .collect::<Vec<_>>()
        .await;

        let scanned_paths = wrap_scan_media_file(&infra, false)
            .await
            .into_iter()
            .map(|result| result.path.to_string())
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_parallel() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(fs.mkrelpaths(
            FS_INDEX,
            50,
            3,
            &SUPPORTED_EXTENSIONS.keys().collect_vec(),
        ))
        .then(move |path| async move { fs.mkfile(FS_INDEX, &path).await })
        .collect::<Vec<_>>()
        .await;

        let scanned_results = wrap_scan_media_file(&infra, true).await;
        let scanned_paths =
            scanned_results.iter().cloned().map(|result| result.path.to_string()).collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }
}
