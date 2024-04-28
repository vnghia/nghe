use std::path::Path;

use flume::Sender;
use ignore::types::TypesBuilder;
use ignore::{DirEntry, Error, WalkBuilder};
use tracing::instrument;

use super::super::song::file_type::SONG_FILE_TYPES;
use crate::utils::path::{AbsolutePath, LocalPath};
use crate::utils::song::file_type::{to_extension, to_glob_pattern};

fn process_dir_entry<P: AsRef<Path>>(
    root: P,
    tx: &Sender<AbsolutePath<LocalPath<'static>>>,
    entry: Result<DirEntry, Error>,
) -> ignore::WalkState {
    match try {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let path = entry.path();
        if metadata.is_file()
            && let Err(e) = tx.send(AbsolutePath::new(
                root.as_ref().to_str().expect("non utf-8 path encountered"),
                path.to_path_buf().into(),
                metadata.into(),
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
pub fn scan_media_files<P: AsRef<Path> + Clone + Send + std::fmt::Debug>(
    root: P,
    tx: Sender<AbsolutePath<LocalPath<'static>>>,
    scan_parallel: bool,
) {
    tracing::info!("start scanning media files");

    let types = match try {
        let mut types = TypesBuilder::new();
        for song_file_type in SONG_FILE_TYPES {
            types.add(to_extension(&song_file_type), to_glob_pattern(&song_file_type))?;
        }
        types.select("all").build()?
    } {
        Ok(r) => r,
        Err::<_, anyhow::Error>(e) => {
            tracing::error!(building_scan_pattern = ?e);
            return;
        }
    };

    if scan_parallel {
        WalkBuilder::new(&root).types(types).build_parallel().run(|| {
            let span = tracing::Span::current();
            let tx = tx.clone();
            let root = root.clone();
            Box::new(move |entry| {
                let _enter = span.enter();
                process_dir_entry(&root, &tx, entry)
            })
        });
    } else {
        for entry in WalkBuilder::new(&root).types(types).build() {
            process_dir_entry(&root, &tx, entry);
        }
    }

    tracing::info!("finish scanning media files");
}
