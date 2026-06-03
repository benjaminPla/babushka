use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::enrollment::{dto::EnrollmentDto, errors::EnrollmentAppError},
    domain::enrollment::repository::EnrollmentRepo,
};

pub struct EnrollmentGetByStudentUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentGetByStudentUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self { Self { enrollment_repo } }

    pub fn execute(&self, student_id: Uuid) -> Result<Vec<EnrollmentDto>, EnrollmentAppError> {
        let enrollments = self.enrollment_repo.get_by_student(student_id)?;
        Ok(enrollments.iter().map(EnrollmentDto::from).collect())
    }
}
