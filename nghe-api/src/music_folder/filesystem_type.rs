use nghe_proc_macro::api_derive;

#[repr(i16)]
#[api_derive(response = false)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilesystemType {
    Local,
    S3,
}
