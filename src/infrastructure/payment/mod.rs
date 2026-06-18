use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use postgres::{Client, Row};
use uuid::Uuid;

use crate::domain::payment::{
    repository::{PaymentRepo, PaymentRepoError},
    value_objects::payment_method::PaymentMethod,
    Payment,
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
    let id:            Uuid           = row.get("id");
    let student_id:    Uuid           = row.get("student_id");
    let amount_cents:  i32            = row.get("amount_cents");
    let method_text:   String         = row.get("payment_method_text");
    let paid_at:       DateTime<Utc>  = row.get("paid_at");
    let notes:         Option<String> = row.get("notes");
    let created_at:    DateTime<Utc>  = row.get("created_at");
    let enrollment_id: Option<Uuid>   = row.get("enrollment_id");

    let payment_method = PaymentMethod::new(&method_text)
        .map_err(|_| PaymentRepoError::Database(format!("unknown payment method: {method_text}")))?;

    Ok(Payment::reconstitute(amount_cents, created_at, enrollment_id, id, notes, paid_at, payment_method, student_id))
}

const SELECT: &str = "
    SELECT p.id, p.student_id, p.amount_cents,
           p.payment_method::text AS payment_method_text,
           p.paid_at, p.notes, p.created_at, p.enrollment_id
    FROM payments p";

impl PaymentRepo for PaymentPgRepo {
    fn create(&self, payment: &Payment) -> Result<(), PaymentRepoError> {
        self.client.lock().unwrap()
            .execute(
                "INSERT INTO payments (id, student_id, amount_cents, payment_method, paid_at, notes, enrollment_id)
                 VALUES ($1, $2, $3, $4::text::payment_method, $5, $6, $7)",
                &[
                    &payment.id(),
                    &payment.student_id(),
                    &payment.amount_cents(),
                    &payment.payment_method(),
                    &payment.paid_at(),
                    &payment.notes(),
                    &payment.enrollment_id(),
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

    fn get_by_student(&self, student_id: Uuid) -> Result<Vec<Payment>, PaymentRepoError> {
        let query = format!("{SELECT} WHERE p.student_id = $1 ORDER BY p.paid_at ASC");
        let rows = self.client.lock().unwrap()
            .query(&query, &[&student_id])
            .map_err(pg_err)?;
        rows.iter().map(row_to_payment).collect()
    }
}
