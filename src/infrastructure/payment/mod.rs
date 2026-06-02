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
    let amount_cents:  i32                   = row.get("amount_cents");
    let due_date:      NaiveDate             = row.get("due_date");
    let paid_at:       Option<DateTime<Utc>> = row.get("paid_at");
    let status:        String                = row.get("status_text");
    let notes:         Option<String>        = row.get("notes");
    let created_at:    DateTime<Utc>         = row.get("created_at");
    let updated_at:    DateTime<Utc>         = row.get("updated_at");

    let status = PaymentStatus::from_db_str(&status)
        .ok_or_else(|| PaymentRepoError::Database(format!("unknown payment status: {status}")))?;

    Ok(Payment::reconstitute(id, enrollment_id, amount_cents, due_date, paid_at, status, notes, created_at, updated_at))
}

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

    fn get_by_enrollment(&self, enrollment_id: Uuid) -> Result<Vec<Payment>, PaymentRepoError> {
        let rows = self.client.lock().unwrap()
            .query(
                "SELECT id, enrollment_id, amount_cents, due_date, paid_at,
                        status::text AS status_text, notes, created_at, updated_at
                 FROM payments WHERE enrollment_id = $1 ORDER BY due_date DESC",
                &[&enrollment_id],
            )
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
