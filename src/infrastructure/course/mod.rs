use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::{
    course::{
        repository::{CourseRepo, CourseRepoError},
        value_objects::{course_capacity::CourseCapacity, course_name::CourseName},
        Course,
    },
    shared::value_objects::{age_group::AgeGroup, cents::Cents, notes::Notes},
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

fn db_err(e: impl ToString) -> CourseRepoError {
    CourseRepoError::Database(e.to_string())
}

fn row_to_course(row: &Row) -> Result<Course, CourseRepoError> {
    let id:                Uuid           = row.get("id");
    let teacher_id:        Uuid           = row.get("teacher_id");
    let name:              String         = row.get("name");
    let age_group:         String         = row.get("age_group_text");
    let capacity:          i16            = row.get("capacity");
    let month_price_cents: i32            = row.get("month_price_cents");
    let class_price_cents: i32            = row.get("class_price_cents");
    let notes:             Option<String> = row.get("notes");
    let created_at:        DateTime<Utc>  = row.get("created_at");
    let updated_at:        DateTime<Utc>  = row.get("updated_at");

    let age_group         = AgeGroup::from_db_str(&age_group).ok_or_else(|| CourseRepoError::Database(format!("unknown age_group: {age_group}")))?;
    let capacity          = CourseCapacity::new(capacity).map_err(db_err)?;
    let class_price_cents = Cents::new(class_price_cents).map_err(db_err)?;
    let month_price_cents = Cents::new(month_price_cents).map_err(db_err)?;
    let name              = CourseName::new(name).map_err(db_err)?;
    let notes             = notes.map(|s| Notes::new(s).map_err(db_err)).transpose()?;

    Ok(Course::reconstitute(age_group, capacity, class_price_cents, created_at, id, month_price_cents, name, notes, teacher_id, updated_at))
}

const SELECT: &str = "
    SELECT id, teacher_id, name, age_group::text AS age_group_text,
           capacity, month_price_cents, class_price_cents, notes,
           created_at, updated_at
    FROM courses";

impl CourseRepo for CoursePgRepo {
    fn create(&self, course: &Course) -> Result<(), CourseRepoError> {
        let notes = course.notes().map(str::to_owned);
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO courses (id, teacher_id, name, age_group, capacity, month_price_cents, class_price_cents, notes)
                 VALUES ($1, $2, $3, $4::text::age_group, $5, $6, $7, $8)",
                &[
                    &course.id(), &course.teacher_id(), &course.name(),
                    &course.age_group().as_db_str(),
                    &course.capacity().value(),
                    &course.month_price_cents().value(),
                    &course.class_price_cents().value(),
                    &notes,
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
        let query = format!("{SELECT} ORDER BY name");
        let rows = self.client.lock().unwrap()
            .query(&query, &[])
            .map_err(pg_err)?;
        rows.iter().map(row_to_course).collect()
    }

    fn get_by_id(&self, id: Uuid) -> Result<Course, CourseRepoError> {
        let query = format!("{SELECT} WHERE id = $1");
        let row = self.client.lock().unwrap()
            .query_opt(&query, &[&id])
            .map_err(pg_err)?
            .ok_or(CourseRepoError::NotFound(id))?;
        row_to_course(&row)
    }

    fn update(&self, course: &Course) -> Result<(), CourseRepoError> {
        let notes = course.notes().map(str::to_owned);
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE courses
                 SET teacher_id = $1, name = $2, age_group = $3::text::age_group,
                     capacity = $4, month_price_cents = $5, class_price_cents = $6, notes = $7
                 WHERE id = $8",
                &[
                    &course.teacher_id(), &course.name(),
                    &course.age_group().as_db_str(),
                    &course.capacity().value(),
                    &course.month_price_cents().value(),
                    &course.class_price_cents().value(),
                    &notes, &course.id(),
                ],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(CourseRepoError::NotFound(course.id())); }
        Ok(())
    }

    fn count_enrollments(&self, course_id: Uuid) -> Result<i64, CourseRepoError> {
        let row = self.client.lock().unwrap()
            .query_one(
                "SELECT COUNT(*) FROM enrollments e
                 JOIN course_periods cp ON cp.id = e.course_period_id
                 WHERE cp.course_id = $1",
                &[&course_id],
            )
            .map_err(pg_err)?;
        Ok(row.get(0))
    }

    fn has_age_group_conflict(&self, course_id: Uuid, age_group: &str) -> Result<bool, CourseRepoError> {
        let row = self.client.lock().unwrap()
            .query_one(
                "SELECT EXISTS (
                    SELECT 1 FROM enrollments e
                    JOIN course_periods cp ON cp.id = e.course_period_id
                    JOIN students s ON s.id = e.student_id
                    WHERE cp.course_id = $1
                      AND s.age_group::text <> $2
                )",
                &[&course_id, &age_group],
            )
            .map_err(pg_err)?;
        Ok(row.get(0))
    }
}
