use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDate, Utc};
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
    let msg = e
        .as_db_error()
        .map(|db| format!("{} (code={})", db.message(), db.code().code()))
        .unwrap_or_else(|| format!("{e:?}"));
    EnrollmentRepoError::Database(msg)
}

fn row_to_enrollment(row: &Row) -> Result<Enrollment, EnrollmentRepoError> {
    let id:               Uuid                   = row.get("id");
    let student_id:       Uuid                   = row.get("student_id");
    let student_name:     String                 = row.get("student_name");
    let course_period_id: Uuid                   = row.get("course_period_id");
    let period_label:     String                 = row.get("period_label");
    let period_end_date:  NaiveDate              = row.get("period_end_date");
    let course_name:      String                 = row.get("course_name");
    let dropped_at:       Option<DateTime<Utc>>  = row.get("dropped_at");
    let latest_payment:   Option<String>         = row.get("latest_payment");
    let enrolled_at:      DateTime<Utc>          = row.get("enrolled_at");
    let updated_at:       DateTime<Utc>          = row.get("updated_at");

    Ok(Enrollment::reconstitute(id, student_id, student_name, course_period_id, period_label, period_end_date, course_name, dropped_at, latest_payment, enrolled_at, updated_at))
}

const SELECT: &str = "
    SELECT e.id, e.student_id, e.course_period_id, e.dropped_at,
           s.first_name || ' ' || s.last_name AS student_name,
           cp.label    AS period_label,
           cp.end_date AS period_end_date,
           c.name      AS course_name,
           e.enrolled_at, e.updated_at,
           p.status::text AS latest_payment
    FROM enrollments e
    JOIN students       s  ON s.id  = e.student_id
    JOIN course_periods cp ON cp.id = e.course_period_id
    JOIN courses        c  ON c.id  = cp.course_id
    LEFT JOIN LATERAL (
        SELECT status FROM payments
        WHERE enrollment_id = e.id
        ORDER BY due_date DESC LIMIT 1
    ) p ON true";

impl EnrollmentRepo for EnrollmentPgRepo {
    fn create(&self, enrollment: &Enrollment) -> Result<(), EnrollmentRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO enrollments (id, student_id, course_period_id)
                 VALUES ($1, $2, $3)",
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

    fn get_all(&self) -> Result<Vec<Enrollment>, EnrollmentRepoError> {
        let query = format!("{SELECT} ORDER BY e.enrolled_at DESC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[])
            .map_err(pg_err)?;
        rows.iter().map(row_to_enrollment).collect()
    }

    fn get_by_student(&self, student_id: Uuid) -> Result<Vec<Enrollment>, EnrollmentRepoError> {
        let query = format!("{SELECT} WHERE e.student_id = $1 ORDER BY e.enrolled_at DESC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&student_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_enrollment).collect()
    }

    fn get_by_id(&self, id: Uuid) -> Result<Enrollment, EnrollmentRepoError> {
        let query = format!("{SELECT} WHERE e.id = $1");
        let row = self.client.lock().unwrap()
            .query_opt(&query, &[&id])
            .map_err(pg_err)?
            .ok_or(EnrollmentRepoError::NotFound(id))?;
        row_to_enrollment(&row)
    }

    fn update(&self, enrollment: &Enrollment) -> Result<(), EnrollmentRepoError> {
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE enrollments SET dropped_at = $1 WHERE id = $2",
                &[&enrollment.dropped_at(), &enrollment.id()],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(EnrollmentRepoError::NotFound(enrollment.id())); }
        Ok(())
    }
}
