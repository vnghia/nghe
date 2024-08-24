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

impl Database {
    pub fn new() -> Self {
        let pool = Pool::builder(Connection::new(env!("DATABASE_URL")))
            .build()
            .expect("Could not build database connection pool");
        Self { pool, key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }
    }
}

impl App {
    pub fn new() -> Self {
        Self { database: Database::new() }
    }
}
