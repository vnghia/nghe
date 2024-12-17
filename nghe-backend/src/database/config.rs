use std::borrow::Cow;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use super::Database;
use crate::orm::configs;
use crate::{Error, error};

pub trait Config {
    const KEY: &'static str;
    const ENCRYPTED: bool;

    fn value(&self) -> Result<Cow<'_, str>, Error>;
}

impl Database {
    pub async fn upsert_config<C: Config>(&self, config: &C) -> Result<(), Error> {
        let value = config.value()?;
        let data = if C::ENCRYPTED {
            let value: &str = value.as_ref();
            configs::Data { text: None, byte: Some(self.encrypt(value).into()) }
        } else {
            configs::Data { text: Some(value), byte: None }
        };
        let upsert = configs::Upsert { key: C::KEY, data };

        diesel::insert_into(configs::table)
            .values(&upsert)
            .on_conflict(configs::key)
            .do_update()
            .set(&upsert)
            .execute(&mut self.get().await?)
            .await?;
        Ok(())
    }

    pub async fn get_config<C: Config>(&self) -> Result<String, Error> {
        let config = configs::table
            .filter(configs::key.eq(C::KEY))
            .select(configs::Data::as_select())
            .get_result(&mut self.get().await?)
            .await?;

        if C::ENCRYPTED {
            String::from_utf8(self.decrypt(
                config.byte.ok_or_else(|| error::Kind::InvalidDatabaseConfigFomat(C::KEY))?,
            )?)
            .map_err(Error::from)
        } else {
            config
                .text
                .ok_or_else(|| error::Kind::InvalidDatabaseConfigFomat(C::KEY).into())
                .map(Cow::into_owned)
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Dummy, Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[derive(Dummy)]
    struct NonEncryptedConfig(String);

    impl Config for NonEncryptedConfig {
        const ENCRYPTED: bool = false;
        const KEY: &'static str = "non-encrypted";

        fn value(&self) -> Result<Cow<'_, str>, Error> {
            Ok((&self.0).into())
        }
    }

    #[derive(Dummy)]
    struct EncryptedConfig(String);

    impl Config for EncryptedConfig {
        const ENCRYPTED: bool = true;
        const KEY: &'static str = "encrypted";

        fn value(&self) -> Result<Cow<'_, str>, Error> {
            Ok((&self.0).into())
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_roundtrip_non_encrypted(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(true, false)] update: bool,
    ) {
        let database = mock.database();

        let config: NonEncryptedConfig = Faker.fake();
        database.upsert_config(&config).await.unwrap();
        let database_config = database.get_config::<NonEncryptedConfig>().await.unwrap();
        assert_eq!(database_config, config.0);

        if update {
            let update: NonEncryptedConfig = Faker.fake();
            database.upsert_config(&update).await.unwrap();
            let database_update = database.get_config::<NonEncryptedConfig>().await.unwrap();
            assert_eq!(database_update, update.0);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_roundtrip_encrypted(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(true, false)] update: bool,
    ) {
        let database = mock.database();

        let config: EncryptedConfig = Faker.fake();
        database.upsert_config(&config).await.unwrap();
        let database_config = database.get_config::<EncryptedConfig>().await.unwrap();
        assert_eq!(database_config, config.0);

        if update {
            let update: EncryptedConfig = Faker.fake();
            database.upsert_config(&update).await.unwrap();
            let database_update = database.get_config::<EncryptedConfig>().await.unwrap();
            assert_eq!(database_update, update.0);
        }
    }
}
