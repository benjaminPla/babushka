use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::{
    shared::value_objects::{
        email::Email,
        first_name::FirstName,
        last_name::LastName,
        notes::Notes,
        phone::Phone,
    },
    student::{
        repository::{StudentRepo, StudentRepoError},
        AgeGroup, Student,
    },
};

pub struct StudentPgRepo {
    client: Arc<Mutex<Client>>,
}

impl StudentPgRepo {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        Self { client }
    }
}

fn pg_err(e: postgres::Error) -> StudentRepoError {
    if let Some(db) = e.as_db_error() {
        if db.code().code() == "23505" {
            let msg = match db.constraint().unwrap_or("") {
                "students_email_unique" => "Ya existe un alumno con ese email",
                _                      => "Ya existe un alumno con esos datos",
            };
            return StudentRepoError::Duplicate(msg.into());
        }
        return StudentRepoError::Database(format!("{} (code={})", db.message(), db.code().code()));
    }
    StudentRepoError::Database(format!("{e:?}"))
}

fn row_to_student(row: &Row) -> Result<Student, StudentRepoError> {
    let id:         Uuid           = row.get("id");
    let first_name: String         = row.get("first_name");
    let last_name:  String         = row.get("last_name");
    let phone:      String         = row.get("phone");
    let email:      Option<String> = row.get("email");
    let notes:      Option<String> = row.get("notes");
    let age_group:  String         = row.get("age_group_text");
    let created_at: DateTime<Utc>  = row.get("created_at");
    let updated_at: DateTime<Utc>  = row.get("updated_at");

    let email      = email.map(|s| Email::new(s).map_err(|e| StudentRepoError::Database(e.to_string()))).transpose()?;
    let first_name = FirstName::new(first_name).map_err(|e| StudentRepoError::Database(e.to_string()))?;
    let last_name  = LastName::new(last_name).map_err(|e| StudentRepoError::Database(e.to_string()))?;
    let notes      = notes.map(|s| Notes::new(s).map_err(|e| StudentRepoError::Database(e.to_string()))).transpose()?;
    let phone      = Phone::new(phone).map_err(|e| StudentRepoError::Database(e.to_string()))?;
    let age_group  = AgeGroup::from_db_str(&age_group)
        .ok_or_else(|| StudentRepoError::Database(format!("unknown age_group: {age_group}")))?;

    Ok(Student::reconstitute(age_group, created_at, email, first_name, id, last_name, notes, phone, updated_at))
}

const SELECT: &str = "SELECT id, first_name, last_name, phone, email, notes,
                             age_group::text AS age_group_text, created_at, updated_at
                      FROM students";

impl StudentRepo for StudentPgRepo {
    fn create(&self, student: &Student) -> Result<(), StudentRepoError> {
        let email = student.email().map(str::to_owned);
        let notes = student.notes().map(str::to_owned);
        self.client
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO students (id, first_name, last_name, phone, email, notes, age_group)
                 VALUES ($1, $2, $3, $4, $5, $6, $7::text::age_group)",
                &[
                    &student.id(),
                    &student.first_name(),
                    &student.last_name(),
                    &student.phone(),
                    &email,
                    &notes,
                    &student.age_group().as_db_str(),
                ],
            )
            .map_err(pg_err)?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<(), StudentRepoError> {
        let n = self.client
            .lock()
            .unwrap()
            .execute("DELETE FROM students WHERE id = $1", &[&id])
            .map_err(pg_err)?;
        if n == 0 { return Err(StudentRepoError::NotFound(id)); }
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Student>, StudentRepoError> {
        let query = format!("{SELECT} ORDER BY first_name, last_name");
        let rows = self.client
            .lock()
            .unwrap()
            .query(&query, &[])
            .map_err(pg_err)?;
        rows.iter().map(row_to_student).collect()
    }

    fn get_by_id(&self, id: Uuid) -> Result<Student, StudentRepoError> {
        let query = format!("{SELECT} WHERE id = $1");
        let row = self.client
            .lock()
            .unwrap()
            .query_opt(&query, &[&id])
            .map_err(pg_err)?
            .ok_or(StudentRepoError::NotFound(id))?;
        row_to_student(&row)
    }

    fn update(&self, student: &Student) -> Result<(), StudentRepoError> {
        let email = student.email().map(str::to_owned);
        let notes = student.notes().map(str::to_owned);
        let n = self.client
            .lock()
            .unwrap()
            .execute(
                "UPDATE students
                 SET first_name = $1, last_name = $2, phone = $3, email = $4,
                     notes = $5, age_group = $6::text::age_group
                 WHERE id = $7",
                &[
                    &student.first_name(),
                    &student.last_name(),
                    &student.phone(),
                    &email,
                    &notes,
                    &student.age_group().as_db_str(),
                    &student.id(),
                ],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(StudentRepoError::NotFound(student.id())); }
        Ok(())
    }
}
