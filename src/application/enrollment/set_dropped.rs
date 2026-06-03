use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::enrollment::errors::EnrollmentAppError,
    domain::enrollment::repository::EnrollmentRepo,
};

pub struct EnrollmentSetDroppedInput {
    pub id:      Uuid,
    pub dropped: bool,
}

pub struct EnrollmentSetDroppedUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentSetDroppedUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self { Self { enrollment_repo } }

    pub fn execute(&self, input: EnrollmentSetDroppedInput) -> Result<(), EnrollmentAppError> {
        let mut enrollment = self.enrollment_repo.get_by_id(input.id)?;
        enrollment.set_dropped(input.dropped);
        self.enrollment_repo.update(&enrollment)?;
        log::info!("[enrollment] dropped={} id={}", input.dropped, input.id);
        Ok(())
    }
}
