use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::enrollment::errors::EnrollmentAppError,
    domain::enrollment::{repository::EnrollmentRepo, EnrollmentStatus},
};

pub struct EnrollmentUpdateStatusInput {
    pub id:     Uuid,
    pub status: EnrollmentStatus,
}

pub struct EnrollmentUpdateStatusUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentUpdateStatusUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self { Self { enrollment_repo } }

    pub fn execute(&self, input: EnrollmentUpdateStatusInput) -> Result<(), EnrollmentAppError> {
        let mut enrollment = self.enrollment_repo.get_by_id(input.id)?;
        let status_label = input.status.as_db_str().to_owned();
        enrollment.set_status(input.status);
        self.enrollment_repo.update(&enrollment)?;
        log::info!("[enrollment] status updated: id={} status={}", input.id, status_label);
        Ok(())
    }
}
