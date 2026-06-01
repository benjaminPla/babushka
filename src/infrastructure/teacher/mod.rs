use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::{
    shared::value_objects::{
        email::Email,
        first_name::FirstName,
        last_name::LastName,
        phone::Phone,
    },
    teacher::{
        repository::{TeacherRepo, TeacherRepoError},
        Teacher,
    },
};

pub struct TeacherPgRepo {
    client: Arc<Mutex<Client>>,
}

impl TeacherPgRepo {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        Self { client }
    }
}

fn row_to_teacher(row: &Row) -> Result<Teacher, TeacherRepoError> {
    let id:         Uuid          = row.get("id");
    let first_name: String        = row.get("first_name");
    let last_name:  String        = row.get("last_name");
    let phone:      String        = row.get("phone");
    let email:      String        = row.get("email");
    let created_at: DateTime<Utc> = row.get("created_at");
    let updated_at: DateTime<Utc> = row.get("updated_at");

    let email      = Email::new(email).map_err(|e| TeacherRepoError::Database(e.to_string()))?;
    let first_name = FirstName::new(first_name).map_err(|e| TeacherRepoError::Database(e.to_string()))?;
    let last_name  = LastName::new(last_name).map_err(|e| TeacherRepoError::Database(e.to_string()))?;
    let phone      = Phone::new(phone).map_err(|e| TeacherRepoError::Database(e.to_string()))?;

    Ok(Teacher::reconstitute(created_at, email, first_name, id, last_name, phone, updated_at))
}

impl TeacherRepo for TeacherPgRepo {
    fn create(&self, teacher: &Teacher) -> Result<(), TeacherRepoError> {
        self.client
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO teachers (id, first_name, last_name, phone, email, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &teacher.id(),
                    &teacher.first_name().value(),
                    &teacher.last_name().value(),
                    &teacher.phone().value(),
                    &teacher.email().value(),
                    &teacher.created_at(),
                    &teacher.updated_at(),
                ],
            )
            .map_err(|e| TeacherRepoError::Database(e.to_string()))?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<(), TeacherRepoError> {
        let n = self.client
            .lock()
            .unwrap()
            .execute("DELETE FROM teachers WHERE id = $1", &[&id])
            .map_err(|e| TeacherRepoError::Database(e.to_string()))?;
        if n == 0 { return Err(TeacherRepoError::NotFound(id)); }
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Teacher>, TeacherRepoError> {
        let rows = self.client
            .lock()
            .unwrap()
            .query(
                "SELECT id, first_name, last_name, phone, email, created_at, updated_at
                 FROM teachers ORDER BY last_name, first_name",
                &[],
            )
            .map_err(|e| TeacherRepoError::Database(e.to_string()))?;
        rows.iter().map(row_to_teacher).collect()
    }

    fn get_by_id(&self, id: Uuid) -> Result<Teacher, TeacherRepoError> {
        let row = self.client
            .lock()
            .unwrap()
            .query_opt(
                "SELECT id, first_name, last_name, phone, email, created_at, updated_at
                 FROM teachers WHERE id = $1",
                &[&id],
            )
            .map_err(|e| TeacherRepoError::Database(e.to_string()))?
            .ok_or(TeacherRepoError::NotFound(id))?;
        row_to_teacher(&row)
    }

    fn update(&self, teacher: &Teacher) -> Result<(), TeacherRepoError> {
        let n = self.client
            .lock()
            .unwrap()
            .execute(
                "UPDATE teachers
                 SET first_name = $1, last_name = $2, phone = $3, email = $4, updated_at = $5
                 WHERE id = $6",
                &[
                    &teacher.first_name().value(),
                    &teacher.last_name().value(),
                    &teacher.phone().value(),
                    &teacher.email().value(),
                    &teacher.updated_at(),
                    &teacher.id(),
                ],
            )
            .map_err(|e| TeacherRepoError::Database(e.to_string()))?;
        if n == 0 { return Err(TeacherRepoError::NotFound(teacher.id())); }
        Ok(())
    }
}
