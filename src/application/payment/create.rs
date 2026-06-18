use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::payment::errors::PaymentAppError;
use crate::domain::payment::repository::PaymentRepo;
use crate::domain::payment::value_objects::payment_method::PaymentMethod;
use crate::domain::payment::Payment;

pub struct PaymentCreateInput {
    pub student_id:     Uuid,
    pub enrollment_id:  Option<Uuid>,
    pub amount_cents:   i32,
    pub payment_method: String,
    pub paid_at:        DateTime<Utc>,
    pub notes:          Option<String>,
}

pub struct PaymentCreateUseCase {
    payment_repo: Arc<dyn PaymentRepo>,
}

impl PaymentCreateUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepo>) -> Self { Self { payment_repo } }

    pub fn execute(&self, input: PaymentCreateInput) -> Result<(), PaymentAppError> {
        if input.amount_cents <= 0 {
            return Err(PaymentAppError::Validation("el monto debe ser mayor a 0".into()));
        }
        let method = PaymentMethod::new(&input.payment_method)
            .map_err(|e| PaymentAppError::Validation(e.to_string()))?;
        let payment = Payment::new(input.amount_cents, input.enrollment_id, input.notes, input.paid_at, method, input.student_id);
        self.payment_repo.create(&payment)?;
        log::info!("[payment] created: id={} student={} method={} paid_at={}",
            payment.id(), input.student_id, input.payment_method, input.paid_at);
        Ok(())
    }
}
