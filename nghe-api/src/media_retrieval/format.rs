use nghe_proc_macro::api_derive;

#[api_derive(request = true)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Raw,
    Aac,
    Flac,
    Mp3,
    Opus,
    Wav,
    Wma,
}
