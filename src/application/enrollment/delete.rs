use std::sync::Arc;

use uuid::Uuid;

use crate::{application::enrollment::errors::EnrollmentAppError, domain::enrollment::repository::EnrollmentRepo};

pub struct EnrollmentDeleteUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentDeleteUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self { Self { enrollment_repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), EnrollmentAppError> {
        self.enrollment_repo.delete(id)?;
        log::info!("[enrollment] deleted: id={}", id);
        Ok(())
    }
}
