use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::student::{AgeGroup, Student};

#[derive(Clone)]
pub struct StudentDto {
    pub age_group:  AgeGroup,
    pub created_at: DateTime<Utc>,
    pub email:      Option<String>,
    pub first_name: String,
    pub id:         Uuid,
    pub last_name:  String,
    pub notes:      Option<String>,
    pub phone:      String,
    pub updated_at: DateTime<Utc>,
}

impl From<&Student> for StudentDto {
    fn from(s: &Student) -> Self {
        Self {
            age_group:  s.age_group(),
            created_at: s.created_at(),
            email:      s.email().map(str::to_owned),
            first_name: s.first_name().to_owned(),
            id:         s.id(),
            last_name:  s.last_name().to_owned(),
            notes:      s.notes().map(str::to_owned),
            phone:      s.phone().to_owned(),
            updated_at: s.updated_at(),
        }
    }
}
