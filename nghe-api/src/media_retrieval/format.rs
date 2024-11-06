use nghe_proc_macro::api_derive;

#[api_derive(request = true)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "test", derive(strum::AsRefStr))]
#[cfg_attr(feature = "test", strum(serialize_all = "lowercase"))]
pub enum Format {
    Raw,
    Aac,
    Flac,
    Mp3,
    Opus,
    Wav,
    Wma,
}
