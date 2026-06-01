use std::sync::Arc;

use uuid::Uuid;

use crate::{application::payment::errors::PaymentAppError, domain::payment::repository::PaymentRepo};

pub struct PaymentMarkPaidUseCase {
    payment_repo: Arc<dyn PaymentRepo>,
}

impl PaymentMarkPaidUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepo>) -> Self { Self { payment_repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), PaymentAppError> {
        self.payment_repo.mark_paid(id)?;
        log::info!("[payment] marked paid: id={}", id);
        Ok(())
    }
}
