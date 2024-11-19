use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use nghe_api::id3::builder::artist as builder;
use uuid::Uuid;

use crate::orm::artists;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}

pub type BuilderSet = builder::SetName<builder::SetId>;

pub type SqlType = sql_types::Record<(sql_types::Uuid, sql_types::Text)>;

impl Required {
    pub fn into_api_builder(self) -> builder::Builder<BuilderSet> {
        id3::artist::Artist::builder().id(self.id).name(self.name)
    }

    pub fn try_into_api(self) -> id3::artist::Artist {
        self.into_api_builder().build()
    }
}

impl FromSql<SqlType, crate::orm::Type> for Required {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let (id, name) = FromSql::<SqlType, crate::orm::Type>::from_sql(bytes)?;
        Ok(Self { id, name })
    }
}
