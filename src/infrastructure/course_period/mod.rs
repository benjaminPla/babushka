use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::course_period::{
    repository::{CoursePeriodRepo, CoursePeriodRepoError},
    CoursePeriod,
};

pub struct CoursePeriodPgRepo {
    client: Arc<Mutex<Client>>,
}

impl CoursePeriodPgRepo {
    pub fn new(client: Arc<Mutex<Client>>) -> Self { Self { client } }
}

fn pg_err(e: postgres::Error) -> CoursePeriodRepoError {
    if let Some(db) = e.as_db_error() {
        if db.code().code() == "23505" {
            let msg = match db.constraint().unwrap_or("") {
                "course_periods_unique" => "Ya existe un período con esa fecha de inicio para este curso",
                _                      => "Ya existe ese período",
            };
            return CoursePeriodRepoError::Duplicate(msg.into());
        }
        return CoursePeriodRepoError::Database(format!("{} (code={})", db.message(), db.code().code()));
    }
    CoursePeriodRepoError::Database(format!("{e:?}"))
}

fn row_to_period(row: &Row) -> Result<CoursePeriod, CoursePeriodRepoError> {
    let id:         Uuid      = row.get("id");
    let course_id:  Uuid      = row.get("course_id");
    let start_date: NaiveDate = row.get("start_date");
    let end_date:   NaiveDate = row.get("end_date");
    let enrolled:   i64       = row.get("enrolled");
    Ok(CoursePeriod::reconstitute(id, course_id, start_date, end_date, enrolled))
}

const SELECT: &str = "
    SELECT cp.id, cp.course_id, cp.start_date, cp.end_date,
           COALESCE(ec.enrolled, 0) AS enrolled
    FROM course_periods cp
    LEFT JOIN (
        SELECT course_period_id, COUNT(*) AS enrolled
        FROM enrollments
        WHERE dropped_at IS NULL
        GROUP BY course_period_id
    ) ec ON ec.course_period_id = cp.id";

impl CoursePeriodRepo for CoursePeriodPgRepo {
    fn create(&self, period: &CoursePeriod) -> Result<(), CoursePeriodRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO course_periods (id, course_id, start_date, end_date)
                 VALUES ($1, $2, $3, $4)",
                &[&period.id(), &period.course_id(), &period.start_date(), &period.end_date()],
            )
            .map_err(pg_err)?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<(), CoursePeriodRepoError> {
        let n = self.client.lock().unwrap()
            .execute("DELETE FROM course_periods WHERE id = $1", &[&id])
            .map_err(pg_err)?;
        if n == 0 { return Err(CoursePeriodRepoError::NotFound(id)); }
        Ok(())
    }

    fn get_by_course(&self, course_id: Uuid) -> Result<Vec<CoursePeriod>, CoursePeriodRepoError> {
        let query = format!("{SELECT} WHERE cp.course_id = $1 ORDER BY cp.start_date DESC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&course_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_period).collect()
    }
}
