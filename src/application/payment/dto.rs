use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::payment::{Payment, PaymentStatus};

#[derive(Clone)]
pub struct PaymentDto {
    pub id:           Uuid,
    pub amount_cents: i32,
    pub due_date:     NaiveDate,
    pub status:       PaymentStatus,
}

impl From<&Payment> for PaymentDto {
    fn from(p: &Payment) -> Self {
        Self {
            id:           p.id(),
            amount_cents: p.amount_cents(),
            due_date:     p.due_date(),
            status:       p.status().clone(),
        }
    }
}
