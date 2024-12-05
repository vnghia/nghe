pub mod form;
pub mod token;

pub use form::Form;
pub use token::Token;

// #[api_derive]
// #[derive(Clone)]
// #[cfg_attr(any(test, feature = "test"), derive(Default))]
// pub struct Auth<'u, 't> {
//     #[serde(rename = "u")]
//     pub username: Cow<'u, str>,
//     #[serde(rename = "s")]
//     pub salt: Cow<'t, str>,
//     #[serde(rename = "t")]
//     pub token: Token,
// }

// impl Auth<'_, '_> {
//     pub fn check(&self, password: impl AsRef<[u8]>) -> bool {
//         let password = password.as_ref();
//         let password_token = Token::new(password, self.salt.as_bytes());
//         password_token == self.token
//     }
// }

// #[cfg(test)]
// mod tests {
//     use fake::faker::internet::en::Password;
//     use fake::Fake;

//     use super::*;

//     #[test]
//     fn test_check_success() {
//         let password = Password(16..32).fake::<String>().into_bytes();
//         let client_salt = Password(8..16).fake::<String>();
//         let client_token = Token::new(&password, &client_salt);
//         assert!(
//             Auth { salt: (&client_salt).into(), token: client_token, ..Default::default() }
//                 .check(password)
//         );
//     }

//     #[test]
//     fn test_check_failed() {
//         let password = Password(16..32).fake::<String>().into_bytes();
//         let client_salt = Password(8..16).fake::<String>();
//         let wrong_client_salt = Password(8..16).fake::<String>();
//         let client_token = Token::new(&password, &client_salt);
//         assert!(
//             !Auth { salt: (&wrong_client_salt).into(), token: client_token, ..Default::default()
// }                 .check(password)
//         );
//     }
// }
