use uuid::Uuid;

use super::date::Date;

#[derive(Debug)]
pub struct Common {
    pub name: String,
    pub date: Date,
    pub release_date: Date,
    pub original_release_date: Date,
    pub mbz_id: Option<Uuid>,
}
