use std::sync::Arc;

use uuid::Uuid;

use crate::{application::enrollment::errors::EnrollmentAppError, domain::enrollment::repository::EnrollmentRepo};

pub struct EnrollmentDeletePaymentUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentDeletePaymentUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self { Self { enrollment_repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), EnrollmentAppError> {
        self.enrollment_repo.delete_payment(id)?;
        log::info!("[enrollment] payment deleted: id={}", id);
        Ok(())
    }
}
