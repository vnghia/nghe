use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use o2o::o2o;
use uuid::Uuid;

use crate::orm::artists;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_into(id3::artist::Required)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}

pub type SqlType = sql_types::Record<(sql_types::Text, sql_types::Uuid)>;

impl FromSql<SqlType, crate::orm::Type> for Required {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let (name, id) = FromSql::<SqlType, crate::orm::Type>::from_sql(bytes)?;
        Ok(Self { id, name })
    }
}
