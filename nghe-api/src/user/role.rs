use nghe_proc_macro::api_derive;

#[api_derive(command = true, fake = true)]
#[derive(Clone, Copy)]
pub struct Role {
    pub admin: bool,
}
