use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDate, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::payment::{
    repository::{PaymentRepo, PaymentRepoError},
    Payment, PaymentStatus,
};

pub struct PaymentPgRepo {
    client: Arc<Mutex<Client>>,
}

impl PaymentPgRepo {
    pub fn new(client: Arc<Mutex<Client>>) -> Self { Self { client } }
}

fn pg_err(e: postgres::Error) -> PaymentRepoError {
    let msg = e
        .as_db_error()
        .map(|db| format!("{} (code={})", db.message(), db.code().code()))
        .unwrap_or_else(|| format!("{e:?}"));
    PaymentRepoError::Database(msg)
}

fn row_to_payment(row: &Row) -> Result<Payment, PaymentRepoError> {
    let id:            Uuid                  = row.get("id");
    let enrollment_id: Uuid                  = row.get("enrollment_id");
    let student_name:  String                = row.get("student_name");
    let course_name:   String                = row.get("course_name");
    let amount_cents:  i32                   = row.get("amount_cents");
    let due_date:      NaiveDate             = row.get("due_date");
    let paid_at:       Option<DateTime<Utc>> = row.get("paid_at");
    let status:        String                = row.get("status_text");
    let notes:         Option<String>        = row.get("notes");
    let created_at:    DateTime<Utc>         = row.get("created_at");
    let updated_at:    DateTime<Utc>         = row.get("updated_at");

    let status = PaymentStatus::from_db_str(&status)
        .ok_or_else(|| PaymentRepoError::Database(format!("unknown payment status: {status}")))?;

    Ok(Payment::reconstitute(id, enrollment_id, student_name, course_name, amount_cents, due_date, paid_at, status, notes, created_at, updated_at))
}

const SELECT: &str = "
    SELECT p.id, p.enrollment_id, p.amount_cents, p.due_date, p.paid_at,
           p.status::text AS status_text, p.notes, p.created_at, p.updated_at,
           s.first_name || ' ' || s.last_name AS student_name,
           c.name AS course_name
    FROM payments p
    JOIN enrollments e ON e.id = p.enrollment_id
    JOIN students    s ON s.id = e.student_id
    JOIN courses     c ON c.id = e.course_id";

impl PaymentRepo for PaymentPgRepo {
    fn create(&self, payment: &Payment) -> Result<(), PaymentRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO payments (id, enrollment_id, amount_cents, due_date, notes)
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    &payment.id(), &payment.enrollment_id(),
                    &payment.amount_cents(), &payment.due_date(), &payment.notes(),
                ],
            )
            .map_err(pg_err)?;
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<(), PaymentRepoError> {
        let n = self.client.lock().unwrap()
            .execute("DELETE FROM payments WHERE id = $1", &[&id])
            .map_err(pg_err)?;
        if n == 0 { return Err(PaymentRepoError::NotFound(id)); }
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Payment>, PaymentRepoError> {
        let query = format!("{SELECT} ORDER BY p.due_date DESC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[])
            .map_err(pg_err)?;
        rows.iter().map(row_to_payment).collect()
    }

    fn get_by_enrollment(&self, enrollment_id: Uuid) -> Result<Vec<Payment>, PaymentRepoError> {
        let query = format!("{SELECT} WHERE p.enrollment_id = $1 ORDER BY p.due_date DESC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&enrollment_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_payment).collect()
    }

    fn mark_paid(&self, id: Uuid) -> Result<(), PaymentRepoError> {
        let n = self.client.lock().unwrap()
            .execute(
                "UPDATE payments SET status = 'paid', paid_at = NOW() WHERE id = $1",
                &[&id],
            )
            .map_err(pg_err)?;
        if n == 0 { return Err(PaymentRepoError::NotFound(id)); }
        Ok(())
    }
}
