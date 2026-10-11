mod bookmarks;
mod browsing;
mod media_retrieval;
mod music_folder;
mod permission;
mod system;
mod user;

nghe_proc_macro::build_command_route! {
    routes = [
        bookmarks,
        browsing,
        media_retrieval,
        music_folder,
        permission,
        system,
        user,
    ],
}
