use diesel_async::pooled_connection::{deadpool, AsyncDieselConnectionManager};
use diesel_async::AsyncPgConnection;

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
