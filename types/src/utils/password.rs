use crate::open_subsonic::common::request::MD5Token;

pub fn to_password_token<P: AsRef<[u8]>, S: AsRef<[u8]>>(password: P, salt: S) -> MD5Token {
    let password = password.as_ref();
    let salt = salt.as_ref();

    let mut data = Vec::with_capacity(password.len() + salt.len());
    data.extend_from_slice(password);
    data.extend_from_slice(salt);
    md5::compute(data).into()
}
