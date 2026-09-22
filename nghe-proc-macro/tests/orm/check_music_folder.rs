use nghe_proc_macro::check_music_folder;

fn test() {
    let artists =
        #[check_music_folder]
        id3::artist::query::with_user_id(user_id).get_results(&mut database.get().await?).await?;
}
