nghe_proc_macro::build_command_action! {
    module = user,
    actions = [create, list(body = false), get(binary = true), delete],
}
