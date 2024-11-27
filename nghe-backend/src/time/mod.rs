use std::time::Duration;

pub async fn now() -> time::OffsetDateTime {
    let now = time::OffsetDateTime::now_utc();
    // Sleep one microsecond because that is the highest time precision postgresql can store.
    tokio::time::sleep(Duration::from_micros(1)).await;
    now
}
