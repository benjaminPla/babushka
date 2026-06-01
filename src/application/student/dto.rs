use uuid::Uuid;

use crate::domain::student::{AgeGroup, Student};

pub struct StudentDto {
    pub id:         Uuid,
    pub age_group:  AgeGroup,
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub notes:      Option<String>,
    pub phone:      String,
}

impl From<&Student> for StudentDto {
    fn from(s: &Student) -> Self {
        Self {
            id:         s.id(),
            age_group:  s.age_group().clone(),
            email:      s.email().value().to_owned(),
            first_name: s.first_name().value().to_owned(),
            last_name:  s.last_name().value().to_owned(),
            notes:      s.notes().map(str::to_owned),
            phone:      s.phone().value().to_owned(),
        }
    }
}
