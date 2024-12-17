use uuid::Uuid;

use crate::Error;
use crate::database::Database;

pub trait Insert {
    async fn insert(&self, database: &Database) -> Result<Uuid, Error>;
}

pub trait Update {
    async fn update(&self, database: &Database, id: Uuid) -> Result<(), Error>;
}

pub trait Upsert: Insert + Update + Sized {
    async fn upsert(
        &self,
        database: &Database,
        id: impl Into<Option<Uuid>>,
    ) -> Result<Uuid, Error> {
        if let Some(id) = id.into() {
            self.update(database, id).await?;
            Ok(id)
        } else {
            self.insert(database).await
        }
    }
}
