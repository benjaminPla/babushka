use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::payment::{Payment, PaymentStatus};

#[derive(Clone)]
pub struct PaymentDto {
    pub id:             Uuid,
    pub student_id:     Uuid,
    pub student_name:   String,
    pub amount_cents:   i32,
    pub discount_cents: i32,
    pub due_date:       NaiveDate,
    pub paid_at:        Option<DateTime<Utc>>,
    pub payment_method: Option<String>,
    pub status:         PaymentStatus,
    pub notes:          Option<String>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

impl From<&Payment> for PaymentDto {
    fn from(p: &Payment) -> Self {
        Self {
            id:             p.id(),
            student_id:     p.student_id(),
            student_name:   p.student_name().to_owned(),
            amount_cents:   p.amount_cents(),
            discount_cents: p.discount_cents(),
            due_date:       p.due_date(),
            paid_at:        p.paid_at(),
            payment_method: p.payment_method().map(str::to_owned),
            status:         p.status().clone(),
            notes:          p.notes().map(str::to_owned),
            created_at:     p.created_at(),
            updated_at:     p.updated_at(),
        }
    }
}
