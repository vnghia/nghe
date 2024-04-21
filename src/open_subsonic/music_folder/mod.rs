mod add_music_folder;
mod get_music_folder_ids;
mod get_music_folder_stat;
mod remove_music_folder;
mod update_music_folder;
mod utils;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(
        get_music_folder_ids,
        get_music_folder_stat,
        add_music_folder,
        update_music_folder,
        remove_music_folder
    )
}

#[cfg(test)]
pub mod test {
    pub use super::add_music_folder::add_music_folder;
}
