use std::sync::Arc;

use uuid::Uuid;

use crate::application::enrollment::errors::EnrollmentAppError;
use crate::domain::enrollment::repository::{EnrollmentRepo, EnrollmentRepoError};
use crate::domain::enrollment::Enrollment;

pub struct EnrollmentCreateInput {
    pub student_id:       Uuid,
    pub course_period_id: Uuid,
}

pub struct EnrollmentCreateUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
}

impl EnrollmentCreateUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>) -> Self {
        Self { enrollment_repo }
    }

    pub fn execute(&self, input: EnrollmentCreateInput) -> Result<(), EnrollmentAppError> {
        let enrollment = Enrollment::new(input.student_id, input.course_period_id);

        self.enrollment_repo.create(&enrollment).map_err(|e| {
            if let EnrollmentRepoError::Database(ref msg) = e {
                if msg.contains("age_group") { return EnrollmentAppError::AgeGroupMismatch; }
                if msg.contains("capacity")  { return EnrollmentAppError::CourseFull; }
            }
            e.into()
        })?;

        log::info!("[enrollment] created: id={} student={} period={}",
            enrollment.id(), input.student_id, input.course_period_id);
        Ok(())
    }
}
