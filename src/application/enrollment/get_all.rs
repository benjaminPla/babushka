use std::sync::Arc;

use crate::{
    application::enrollment::{dto::EnrollmentDto, errors::EnrollmentAppError},
    domain::enrollment::repository::EnrollmentRepo,
};

pub struct EnrollmentGetAllUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentGetAllUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self { Self { enrollment_repo } }

    pub fn execute(&self) -> Result<Vec<EnrollmentDto>, EnrollmentAppError> {
        let enrollments = self.enrollment_repo.get_all()?;
        Ok(enrollments.iter().map(EnrollmentDto::from).collect())
    }
}
