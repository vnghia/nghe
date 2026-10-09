nghe_proc_macro::build_command! {
    module = system,
    actions = [
        get_open_subsonic_extensions(body = false),
        health(body = false),
        ping(body = false),
    ],
}
