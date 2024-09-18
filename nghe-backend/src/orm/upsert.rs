use uuid::Uuid;

use crate::database::Database;
use crate::Error;

pub trait Trait: Sized {
    async fn insert(self, database: &Database) -> Result<Uuid, Error>;
    async fn update(self, database: &Database, id: Uuid) -> Result<(), Error>;

    async fn upsert(self, database: &Database, id: impl Into<Option<Uuid>>) -> Result<Uuid, Error> {
        if let Some(id) = id.into() {
            self.update(database, id).await?;
            Ok(id)
        } else {
            self.insert(database).await
        }
    }
}
