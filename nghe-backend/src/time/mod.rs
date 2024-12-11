use std::time::Duration;

pub async fn now() -> time::OffsetDateTime {
    let now = time::OffsetDateTime::now_utc();
    // Sleep one microsecond because that is the highest time precision postgresql can store.
    // In test, we will sleep a little bit more just to be sure.
    tokio::time::sleep(Duration::from_micros(if cfg!(test) { 10 } else { 1 })).await;
    now
}
