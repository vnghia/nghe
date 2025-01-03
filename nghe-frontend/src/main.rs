use leptos::mount::mount_to_body;
use nghe_frontend::Body;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(Body);
}
