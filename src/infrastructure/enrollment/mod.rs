use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::enrollment::{
    repository::{EnrollmentRepo, EnrollmentRepoError},
    Enrollment,
};

pub struct EnrollmentPgRepo {
    client: Arc<Mutex<Client>>,
}

impl EnrollmentPgRepo {
    pub fn new(client: Arc<Mutex<Client>>) -> Self { Self { client } }
}

fn pg_err(e: postgres::Error) -> EnrollmentRepoError {
    if let Some(db) = e.as_db_error() {
        if db.code().code() == "23505" {
            let msg = match db.constraint().unwrap_or("") {
                "enrollments_unique" => "El alumno ya está inscrito en ese período",
                _                   => "Ya existe esa inscripción",
            };
            return EnrollmentRepoError::Duplicate(msg.into());
        }
        return EnrollmentRepoError::Database(format!("{} (code={})", db.message(), db.code().code()));
    }
    EnrollmentRepoError::Database(format!("{e:?}"))
}

fn row_to_enrollment(row: &Row) -> Result<Enrollment, EnrollmentRepoError> {
    let id:           Uuid          = row.get("id");
    let student_id:   Uuid          = row.get("student_id");
    let period_id:    Uuid          = row.get("course_period_id");
    let course_id:    Uuid          = row.get("course_id");
    let period_label: String        = row.get("period_label");
    let course_name:  String        = row.get("course_name");
    let enrolled_at:  DateTime<Utc> = row.get("enrolled_at");

    Ok(Enrollment::reconstitute(id, student_id, period_id, course_id, period_label, course_name, enrolled_at))
}

const SELECT: &str = "
    SELECT e.id, e.student_id, e.course_period_id, e.enrolled_at,
           c.id AS course_id,
           TO_CHAR(cp.start_date, 'FMMonth YYYY') AS period_label,
           c.name AS course_name
    FROM enrollments e
    JOIN course_periods cp ON cp.id = e.course_period_id
    JOIN courses        c  ON c.id  = cp.course_id";

impl EnrollmentRepo for EnrollmentPgRepo {
    fn create(&self, enrollment: &Enrollment) -> Result<(), EnrollmentRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO enrollments (id, student_id, course_period_id) VALUES ($1, $2, $3)",
                &[&enrollment.id(), &enrollment.student_id(), &enrollment.course_period_id()],
            )
            .map_err(pg_err)?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<(), EnrollmentRepoError> {
        let n = self.client.lock().unwrap()
            .execute("DELETE FROM enrollments WHERE id = $1", &[&id])
            .map_err(pg_err)?;
        if n == 0 { return Err(EnrollmentRepoError::NotFound(id)); }
        Ok(())
    }

    fn get_by_student(&self, student_id: Uuid) -> Result<Vec<Enrollment>, EnrollmentRepoError> {
        let query = format!("{SELECT} WHERE e.student_id = $1 ORDER BY e.enrolled_at DESC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&student_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_enrollment).collect()
    }
}
