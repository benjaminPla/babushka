use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::teacher::Teacher;

#[derive(Clone)]
pub struct TeacherDto {
    pub created_at: DateTime<Utc>,
    pub email:      Option<String>,
    pub first_name: String,
    pub id:         Uuid,
    pub last_name:  String,
    pub notes:      Option<String>,
    pub phone:      String,
    pub updated_at: DateTime<Utc>,
}

impl From<&Teacher> for TeacherDto {
    fn from(t: &Teacher) -> Self {
        Self {
            created_at: t.created_at(),
            email:      t.email().map(str::to_owned),
            first_name: t.first_name().to_owned(),
            id:         t.id(),
            last_name:  t.last_name().to_owned(),
            notes:      t.notes().map(str::to_owned),
            phone:      t.phone().to_owned(),
            updated_at: t.updated_at(),
        }
    }
}
