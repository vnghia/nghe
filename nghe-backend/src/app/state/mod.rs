use diesel_async::pooled_connection::{deadpool, AsyncDieselConnectionManager};
use diesel_async::AsyncPgConnection;
use libaes::Cipher;

use super::error::Error;

type Connection = AsyncDieselConnectionManager<AsyncPgConnection>;
type Pool = deadpool::Pool<AsyncPgConnection>;

pub type Key = [u8; libaes::AES_128_KEY_LEN];

#[derive(Clone)]
pub struct Database {
    pool: Pool,
    key: Key,
}

#[derive(Clone)]
pub struct App {
    pub database: Database,
}

impl Database {
    const IV_LEN: usize = 16;

    pub fn new() -> Self {
        let pool = Pool::builder(Connection::new(env!("DATABASE_URL")))
            .build()
            .expect("Could not build database connection pool");
        Self { pool, key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }
    }

    pub async fn get(&self) -> Result<deadpool::Object<AsyncPgConnection>, Error> {
        self.pool.get().await.map_err(|_| Error::CheckoutConnectionPool)
    }

    pub fn decrypt(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        Self::decrypt_impl(self.key, data)
    }

    fn decrypt_impl(key: Key, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        let cipher_text = &data[Self::IV_LEN..];
        let iv = &data[..Self::IV_LEN];

        let output = Cipher::new_128(&key).cbc_decrypt(iv, cipher_text);
        if output.is_empty() { Err(Error::DecryptDatabaseValue) } else { Ok(output) }
    }
}

impl App {
    pub fn new() -> Self {
        Self { database: Database::new() }
    }
}
