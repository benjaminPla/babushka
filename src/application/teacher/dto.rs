use uuid::Uuid;

use crate::domain::teacher::Teacher;

pub struct TeacherDto {
    pub id:         Uuid,
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub phone:      String,
}

impl From<&Teacher> for TeacherDto {
    fn from(t: &Teacher) -> Self {
        Self {
            id:         t.id(),
            email:      t.email().value().to_owned(),
            first_name: t.first_name().value().to_owned(),
            last_name:  t.last_name().value().to_owned(),
            phone:      t.phone().value().to_owned(),
        }
    }
}
