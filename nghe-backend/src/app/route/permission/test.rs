use diesel::QueryDsl;
use diesel_async::RunQueryDsl;

use crate::orm::user_music_folder_permissions;
use crate::test::Mock;

pub async fn reset(mock: &Mock) {
    diesel::delete(user_music_folder_permissions::table)
        .execute(&mut mock.get().await)
        .await
        .unwrap();
}

pub async fn count(mock: &Mock) -> usize {
    user_music_folder_permissions::table
        .count()
        .get_result::<i64>(&mut mock.get().await)
        .await
        .unwrap()
        .try_into()
        .unwrap()
}
