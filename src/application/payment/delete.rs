use std::sync::Arc;

use uuid::Uuid;

use crate::application::payment::errors::PaymentAppError;
use crate::domain::payment::repository::PaymentRepo;

pub struct PaymentDeleteUseCase {
    payment_repo: Arc<dyn PaymentRepo>,
}

impl PaymentDeleteUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepo>) -> Self { Self { payment_repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), PaymentAppError> {
        self.payment_repo.delete(id)?;
        log::info!("[payment] deleted: id={}", id);
        Ok(())
    }
}
