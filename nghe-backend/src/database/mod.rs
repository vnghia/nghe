mod config;

use diesel_async::pooled_connection::{deadpool, AsyncDieselConnectionManager};
use diesel_async::AsyncPgConnection;
use libaes::Cipher;

use crate::Error;

type Connection = AsyncDieselConnectionManager<AsyncPgConnection>;
type Pool = deadpool::Pool<AsyncPgConnection>;

pub type Key = [u8; libaes::AES_128_KEY_LEN];

#[derive(Clone)]
pub struct Database {
    pool: Pool,
    key: Key,
}

impl Database {
    const IV_LEN: usize = 16;

    pub fn new(config: &crate::config::Database) -> Self {
        let pool = Pool::builder(Connection::new(&config.url))
            .build()
            .expect("Could not build database connection pool");
        Self { pool, key: config.key }
    }

    pub async fn get(&self) -> Result<deadpool::Object<AsyncPgConnection>, Error> {
        self.pool.get().await.map_err(|_| Error::CheckoutConnectionPool)
    }

    pub fn encrypt(&self, data: impl AsRef<[u8]>) -> Vec<u8> {
        Self::encrypt_impl(&self.key, data)
    }

    pub fn decrypt(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        Self::decrypt_impl(&self.key, data)
    }

    fn encrypt_impl(key: &Key, data: impl AsRef<[u8]>) -> Vec<u8> {
        let data = data.as_ref();

        let iv: [u8; Self::IV_LEN] = rand::random();
        [iv.as_slice(), Cipher::new_128(key).cbc_encrypt(&iv, data).as_slice()].concat()
    }

    fn decrypt_impl(key: &Key, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        let cipher_text = &data[Self::IV_LEN..];
        let iv = &data[..Self::IV_LEN];

        let output = Cipher::new_128(key).cbc_decrypt(iv, cipher_text);
        if output.is_empty() { Err(Error::DecryptDatabaseValue) } else { Ok(output) }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn test_roundtrip() {
        let key: Key = Faker.fake();
        let data = (16..32).fake::<String>().into_bytes();
        assert_eq!(
            data,
            Database::decrypt_impl(&key, Database::encrypt_impl(&key, &data)).unwrap()
        );
    }
}
