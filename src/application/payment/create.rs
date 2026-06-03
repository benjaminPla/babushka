use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    application::payment::errors::PaymentAppError,
    domain::payment::{repository::PaymentRepo, Payment},
};

pub struct PaymentCreateInput {
    pub student_id:   Uuid,
    pub amount_cents: i32,
    pub due_date:     NaiveDate,
    pub notes:        Option<String>,
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
        let payment = Payment::new(input.student_id, input.amount_cents, input.due_date, input.notes);
        self.payment_repo.create(&payment)?;
        log::info!("[payment] created: id={} student={} due={}", payment.id(), input.student_id, input.due_date);
        Ok(())
    }
}
