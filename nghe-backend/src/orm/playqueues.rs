use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_derives::AsChangeset;
use o2o::o2o;
use uuid::Uuid;

use crate::Error;
pub use crate::schema::playqueues::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[diesel(table_name = playqueues, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[try_from_owned(nghe_api::bookmarks::save_playqueue::Request, Error)]
pub struct Data {
    #[diesel(select_expression = sql("playqueues.ids ids"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Array<sql_types::Uuid>>)]
    pub ids: Vec<Uuid>,
    pub current: Option<Uuid>,
    #[from(~.map(i64::try_from).transpose()?)]
    pub position: Option<i64>,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Data, playqueues};
    use crate::Error;
    use crate::database::Database;

    impl crate::orm::upsert::Update for Data {
        async fn update(&self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::insert_into(playqueues::table)
                .values((playqueues::user_id.eq(id), self))
                .on_conflict(playqueues::user_id)
                .do_update()
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
