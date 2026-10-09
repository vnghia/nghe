use nghe_proc_macro::api_derive;

#[api_derive]
#[endpoint(path = "getOpenSubsonicExtensions")]
pub struct Request;

#[api_derive]
#[derive(Clone, Copy)]
pub struct Extension {
    pub name: &'static str,
    pub versions: &'static [u8],
}

static EXTENSIONS: [Extension; 4] = [
    Extension { name: "transcodeOffset", versions: &[1] },
    Extension { name: "songLyrics", versions: &[1] },
    Extension { name: "formPost", versions: &[1] },
    Extension { name: "apiKeyAuthentication", versions: &[1] },
];

#[api_derive]
pub struct Response {
    #[serde(serialize_with = "serde::emit_extensions")]
    pub open_subsonic_extensions: (),
}

mod serde {
    use ::serde::Serializer;
    use ::serde::ser::SerializeSeq as _;

    use super::EXTENSIONS;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn emit_extensions<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(EXTENSIONS.len()))?;
        for extension in EXTENSIONS {
            seq.serialize_element(&extension)?;
        }
        seq.end()
    }
}
