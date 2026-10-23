mod bookmarks;
mod browsing;
mod key;
mod lists;
mod media_annotation;
mod media_retrieval;
mod music_folder;
mod permission;
mod playlist;
mod scan;
mod search;
mod system;
mod user;

nghe_proc_macro::build_command_route! {
    routes = [
        bookmarks,
        browsing,
        key,
        lists,
        media_annotation,
        media_retrieval,
        music_folder,
        permission,
        playlist,
        scan,
        search,
        system,
        user,
    ],
}
