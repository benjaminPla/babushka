use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDate, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::enrollment::{
    repository::{EnrollmentRepo, EnrollmentRepoError},
    value_objects::{
        payment_method::PaymentMethod,
        pricing_type::PricingType,
    },
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
    let id:                Uuid                  = row.get("id");
    let student_id:        Uuid                  = row.get("student_id");
    let period_id:         Uuid                  = row.get("course_period_id");
    let course_id:         Uuid                  = row.get("course_id");
    let period_label:      String                = row.get("period_label");
    let course_name:       String                = row.get("course_name");
    let student_name:      String                = row.get("student_name");
    let pricing_type_str:  String                = row.get("pricing_type");
    let paid_amount_cents: Option<i32>           = row.get("paid_amount_cents");
    let method_text:       Option<String>        = row.get("payment_method_text");
    let paid_at:           Option<DateTime<Utc>> = row.get("paid_at");
    let payment_notes:     Option<String>        = row.get("payment_notes");

    let pricing_type = PricingType::new(&pricing_type_str)
        .map_err(|_| EnrollmentRepoError::Database(format!("unknown pricing_type: {pricing_type_str}")))?;

    let payment_method = match method_text {
        Some(m) => Some(PaymentMethod::new(&m)
            .map_err(|_| EnrollmentRepoError::Database(format!("unknown payment_method: {m}")))?),
        None => None,
    };

    Ok(Enrollment::reconstitute(
        id, student_id, period_id, course_id, period_label, course_name, student_name,
        pricing_type, paid_amount_cents, payment_method, paid_at, payment_notes,
    ))
}

const SELECT: &str = "
    SELECT e.id, e.student_id, e.course_period_id,
           e.pricing_type, e.paid_amount_cents,
           e.payment_method::text AS payment_method_text,
           e.paid_at, e.payment_notes,
           c.id AS course_id,
           TO_CHAR(cp.start_date, 'FMMonth YYYY') AS period_label,
           c.name AS course_name,
           s.first_name || ' ' || s.last_name AS student_name
    FROM enrollments e
    JOIN course_periods cp ON cp.id = e.course_period_id
    JOIN courses        c  ON c.id  = cp.course_id
    JOIN students       s  ON s.id  = e.student_id";

impl EnrollmentRepo for EnrollmentPgRepo {
    fn create(&self, enrollment: &Enrollment) -> Result<(), EnrollmentRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO enrollments (id, student_id, course_period_id, pricing_type)
                 VALUES ($1, $2, $3, $4)",
                &[
                    &enrollment.id(),
                    &enrollment.student_id(),
                    &enrollment.course_period_id(),
                    &enrollment.pricing_type(),
                ],
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

    fn get_by_course(&self, course_id: Uuid) -> Result<Vec<Enrollment>, EnrollmentRepoError> {
        let query = format!("{SELECT} WHERE c.id = $1 ORDER BY s.last_name, s.first_name");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&course_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_enrollment).collect()
    }

    fn sum_paid_between(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<i32, EnrollmentRepoError> {
        let row = self.client.lock().unwrap()
            .query_one(
                "SELECT COALESCE(SUM(paid_amount_cents), 0)::int AS total
                 FROM enrollments
                 WHERE paid_at >= $1 AND paid_at < $2",
                &[&from, &to],
            )
            .map_err(pg_err)?;
        Ok(row.get::<_, i32>("total"))
    }

    fn sum_expected_in_month(&self, from: NaiveDate, to: NaiveDate) -> Result<i32, EnrollmentRepoError> {
        let row = self.client.lock().unwrap()
            .query_one(
                "SELECT COALESCE(SUM(
                     CASE WHEN e.pricing_type = 'monthly'
                          THEN c.month_price_cash_cents
                          ELSE c.class_price_cash_cents
                     END
                 ), 0)::int AS total
                 FROM enrollments e
                 JOIN course_periods cp ON cp.id = e.course_period_id
                 JOIN courses        c  ON c.id  = cp.course_id
                 WHERE cp.start_date >= $1 AND cp.start_date < $2",
                &[&from, &to],
            )
            .map_err(pg_err)?;
        Ok(row.get::<_, i32>("total"))
    }

    fn pay(&self, id: Uuid, amount_cents: i32, method: PaymentMethod, paid_at: DateTime<Utc>, notes: Option<String>) -> Result<(), EnrollmentRepoError> {
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE enrollments
                 SET paid_amount_cents = $2,
                     payment_method    = $3::text::payment_method,
                     paid_at           = $4,
                     payment_notes     = $5
                 WHERE id = $1",
                &[&id, &amount_cents, &method.value(), &paid_at, &notes],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(EnrollmentRepoError::NotFound(id)); }
        Ok(())
    }

    fn delete_payment(&self, id: Uuid) -> Result<(), EnrollmentRepoError> {
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE enrollments
                 SET paid_amount_cents = NULL,
                     payment_method    = NULL,
                     paid_at           = NULL,
                     payment_notes     = NULL
                 WHERE id = $1",
                &[&id],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(EnrollmentRepoError::NotFound(id)); }
        Ok(())
    }
}
