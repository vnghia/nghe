pub mod add;
pub mod remove;
pub mod update;

use nghe_proc_macro::api_derive;

#[api_derive(command = true)]
#[derive(Clone, Copy, Default)]
pub struct Permission {
    pub owner: bool,
    pub share: bool,
}
