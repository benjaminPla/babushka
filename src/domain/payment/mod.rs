pub mod repository;
pub mod value_objects;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::payment::value_objects::payment_method::PaymentMethod;

pub struct Payment {
    amount_cents:   i32,
    enrollment_id:  Option<Uuid>,
    id:             Uuid,
    notes:          Option<String>,
    paid_at:        DateTime<Utc>,
    payment_method: PaymentMethod,
    student_id:     Uuid,
}

impl Payment {
    pub fn new(
        amount_cents:   i32,
        enrollment_id:  Option<Uuid>,
        notes:          Option<String>,
        paid_at:        DateTime<Utc>,
        payment_method: PaymentMethod,
        student_id:     Uuid,
    ) -> Self {
        Self { amount_cents, enrollment_id, id: Uuid::new_v4(), notes, paid_at, payment_method, student_id }
    }

    pub fn reconstitute(
        amount_cents:   i32,
        _created_at:    DateTime<Utc>,
        enrollment_id:  Option<Uuid>,
        id:             Uuid,
        notes:          Option<String>,
        paid_at:        DateTime<Utc>,
        payment_method: PaymentMethod,
        student_id:     Uuid,
    ) -> Self {
        Self { amount_cents, enrollment_id, id, notes, paid_at, payment_method, student_id }
    }

    pub fn amount_cents(&self)   -> i32           { self.amount_cents }
    pub fn enrollment_id(&self)  -> Option<Uuid>  { self.enrollment_id }
    pub fn id(&self)             -> Uuid          { self.id }
    pub fn notes(&self)          -> Option<&str>  { self.notes.as_deref() }
    pub fn paid_at(&self)        -> DateTime<Utc> { self.paid_at }
    pub fn payment_method(&self) -> &str          { self.payment_method.value() }
    pub fn student_id(&self)     -> Uuid          { self.student_id }
}
