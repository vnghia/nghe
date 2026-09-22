use nghe_proc_macro::check_music_folder;
fn test() {
    let artists = if let Some(music_folder_ids) = request.music_folder_ids.as_ref() {
        id3::artist::query::with_music_folder(user_id, music_folder_ids)
            .get_results(&mut database.get().await?)
            .await?
    } else {
        id3::artist::query::with_user_id(user_id)
            .get_results(&mut database.get().await?)
            .await?
    };
}
