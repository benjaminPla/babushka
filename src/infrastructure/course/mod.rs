use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::{
    course::{repository::{CourseRepo, CourseRepoError}, Course},
    shared::value_objects::age_group::AgeGroup,
};

pub struct CoursePgRepo {
    client: Arc<Mutex<Client>>,
}

impl CoursePgRepo {
    pub fn new(client: Arc<Mutex<Client>>) -> Self { Self { client } }
}

fn pg_err(e: postgres::Error) -> CourseRepoError {
    let msg = e
        .as_db_error()
        .map(|db| format!("{} (code={})", db.message(), db.code().code()))
        .unwrap_or_else(|| format!("{e:?}"));
    CourseRepoError::Database(msg)
}

fn row_to_course(row: &Row) -> Result<Course, CourseRepoError> {
    let id:                Uuid           = row.get("id");
    let teacher_id:        Uuid           = row.get("teacher_id");
    let teacher_name:      String         = row.get("teacher_name");
    let name:              String         = row.get("name");
    let age_group:         String         = row.get("age_group_text");
    let capacity:          i16            = row.get("capacity");
    let price_cents:       i32            = row.get("price_cents");
    let class_price_cents: i32            = row.get("class_price_cents");
    let notes:             Option<String> = row.get("notes");
    let created_at:        DateTime<Utc>  = row.get("created_at");
    let updated_at:        DateTime<Utc>  = row.get("updated_at");

    let age_group = AgeGroup::from_db_str(&age_group)
        .ok_or_else(|| CourseRepoError::Database(format!("unknown age_group: {age_group}")))?;

    Ok(Course::reconstitute(id, teacher_id, teacher_name, name, age_group, capacity, price_cents, class_price_cents, notes, created_at, updated_at))
}

const SELECT: &str = "
    SELECT c.id, c.teacher_id,
           t.first_name || ' ' || t.last_name AS teacher_name,
           c.name, c.age_group::text AS age_group_text,
           c.capacity, c.price_cents, c.class_price_cents, c.notes,
           c.created_at, c.updated_at
    FROM courses c
    JOIN teachers t ON t.id = c.teacher_id";

impl CourseRepo for CoursePgRepo {
    fn create(&self, course: &Course) -> Result<(), CourseRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO courses (id, teacher_id, name, age_group, capacity, price_cents, class_price_cents, notes)
                 VALUES ($1, $2, $3, $4::text::age_group, $5, $6, $7, $8)",
                &[
                    &course.id(), &course.teacher_id(), &course.name(),
                    &course.age_group().as_db_str(), &course.capacity(),
                    &course.price_cents(), &course.class_price_cents(), &course.notes(),
                ],
            )
            .map_err(pg_err)?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<(), CourseRepoError> {
        let n = self.client.lock().unwrap()
            .execute("DELETE FROM courses WHERE id = $1", &[&id])
            .map_err(pg_err)?;
        if n == 0 { return Err(CourseRepoError::NotFound(id)); }
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Course>, CourseRepoError> {
        let query = format!("{SELECT} ORDER BY c.name");
        let rows = self.client.lock().unwrap()
            .query(&query, &[])
            .map_err(pg_err)?;
        rows.iter().map(row_to_course).collect()
    }

    fn get_by_id(&self, id: Uuid) -> Result<Course, CourseRepoError> {
        let query = format!("{SELECT} WHERE c.id = $1");
        let row = self.client.lock().unwrap()
            .query_opt(&query, &[&id])
            .map_err(pg_err)?
            .ok_or(CourseRepoError::NotFound(id))?;
        row_to_course(&row)
    }

    fn update(&self, course: &Course) -> Result<(), CourseRepoError> {
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE courses
                 SET teacher_id = $1, name = $2, age_group = $3::text::age_group,
                     capacity = $4, price_cents = $5, class_price_cents = $6, notes = $7
                 WHERE id = $8",
                &[
                    &course.teacher_id(), &course.name(),
                    &course.age_group().as_db_str(), &course.capacity(),
                    &course.price_cents(), &course.class_price_cents(),
                    &course.notes(), &course.id(),
                ],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(CourseRepoError::NotFound(course.id())); }
        Ok(())
    }
}
