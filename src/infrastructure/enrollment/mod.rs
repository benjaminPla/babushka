use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::enrollment::{
    repository::{EnrollmentRepo, EnrollmentRepoError},
    Enrollment, EnrollmentStatus,
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
    let id:             Uuid           = row.get("id");
    let student_id:     Uuid           = row.get("student_id");
    let student_name:   String         = row.get("student_name");
    let course_id:      Uuid           = row.get("course_id");
    let status:         String         = row.get("status_text");
    let latest_payment: Option<String> = row.get("latest_payment");
    let enrolled_at:    DateTime<Utc>  = row.get("enrolled_at");
    let updated_at:     DateTime<Utc>  = row.get("updated_at");

    let status = EnrollmentStatus::from_db_str(&status)
        .ok_or_else(|| EnrollmentRepoError::Database(format!("unknown enrollment status: {status}")))?;

    Ok(Enrollment::reconstitute(id, student_id, student_name, course_id, status, latest_payment, enrolled_at, updated_at))
}

const SELECT: &str = "
    SELECT e.id, e.student_id, e.course_id,
           s.first_name || ' ' || s.last_name AS student_name,
           e.status::text AS status_text,
           e.enrolled_at, e.updated_at,
           p.status::text AS latest_payment
    FROM enrollments e
    JOIN students s ON s.id = e.student_id
    LEFT JOIN LATERAL (
        SELECT status FROM payments
        WHERE enrollment_id = e.id
        ORDER BY due_date DESC LIMIT 1
    ) p ON true";

impl EnrollmentRepo for EnrollmentPgRepo {
    fn create(&self, enrollment: &Enrollment) -> Result<(), EnrollmentRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO enrollments (id, student_id, course_id)
                 VALUES ($1, $2, $3)",
                &[&enrollment.id(), &enrollment.student_id(), &enrollment.course_id()],
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

    fn get_by_course(&self, course_id: Uuid) -> Result<Vec<Enrollment>, EnrollmentRepoError> {
        let query = format!("{SELECT} WHERE e.course_id = $1 ORDER BY s.last_name, s.first_name");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&course_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_enrollment).collect()
    }

    fn get_by_id(&self, id: Uuid) -> Result<Enrollment, EnrollmentRepoError> {
        let query = format!("{SELECT} WHERE e.id = $1");
        let row = self.client.lock().unwrap()
            .query_opt(&query, &[&id])
            .map_err(|e| EnrollmentRepoError::Database(e.to_string()))?
            .ok_or(EnrollmentRepoError::NotFound(id))?;
        row_to_enrollment(&row)
    }

    fn count_active(&self, course_id: Uuid) -> Result<i64, EnrollmentRepoError> {
        let row = self.client.lock().unwrap()
            .query_one(
                "SELECT COUNT(*) FROM enrollments WHERE course_id = $1 AND status = 'active'",
                &[&course_id],
            )
            .map_err(pg_err)?;
        Ok(row.get(0))
    }

    fn update(&self, enrollment: &Enrollment) -> Result<(), EnrollmentRepoError> {
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE enrollments SET status = $1::enrollment_status WHERE id = $2",
                &[&enrollment.status().as_db_str(), &enrollment.id()],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(EnrollmentRepoError::NotFound(enrollment.id())); }
        Ok(())
    }
}
