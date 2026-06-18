use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::enrollment::errors::EnrollmentAppError;
use crate::domain::enrollment::repository::EnrollmentRepo;
use crate::domain::enrollment::value_objects::payment_method::PaymentMethod;

pub struct EnrollmentPayInput {
    pub enrollment_id:  Uuid,
    pub amount_cents:   i32,
    pub payment_method: String,
    pub paid_at:        DateTime<Utc>,
}

pub struct EnrollmentPayUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentPayUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self {
        Self { enrollment_repo }
    }

    pub fn execute(&self, input: EnrollmentPayInput) -> Result<(), EnrollmentAppError> {
        if input.amount_cents <= 0 {
            return Err(EnrollmentAppError::Validation("El monto debe ser mayor a cero".into()));
        }

        let method = PaymentMethod::new(&input.payment_method)
            .map_err(|e| EnrollmentAppError::Validation(e.to_string()))?;

        self.enrollment_repo
            .pay(input.enrollment_id, input.amount_cents, method, input.paid_at)
            .map_err(EnrollmentAppError::from)?;

        log::info!("[enrollment] paid: id={} amount={}", input.enrollment_id, input.amount_cents);
        Ok(())
    }
}
