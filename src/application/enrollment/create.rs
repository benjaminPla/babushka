use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::enrollment::errors::EnrollmentAppError,
    domain::{
        course::repository::CourseRepo,
        course_period::repository::CoursePeriodRepo,
        enrollment::{repository::{EnrollmentRepo, EnrollmentRepoError}, Enrollment},
    },
};

pub struct EnrollmentCreateInput {
    pub student_id:       Uuid,
    pub course_period_id: Uuid,
}

pub struct EnrollmentCreateUseCase {
    enrollment_repo:    Arc<dyn EnrollmentRepo>,
    course_period_repo: Arc<dyn CoursePeriodRepo>,
    course_repo:        Arc<dyn CourseRepo>,
}

impl EnrollmentCreateUseCase {
    pub fn new(
        enrollment_repo:    Arc<dyn EnrollmentRepo>,
        course_period_repo: Arc<dyn CoursePeriodRepo>,
        course_repo:        Arc<dyn CourseRepo>,
    ) -> Self {
        Self { enrollment_repo, course_period_repo, course_repo }
    }

    pub fn execute(&self, input: EnrollmentCreateInput) -> Result<(), EnrollmentAppError> {
        let period = self.course_period_repo.get_by_id(input.course_period_id)
            .map_err(|_| EnrollmentAppError::NotFound)?;
        let course = self.course_repo.get_by_id(period.course_id())
            .map_err(|_| EnrollmentAppError::NotFound)?;

        let enrollment = Enrollment::new(input.student_id, input.course_period_id, course.price_cents());
        self.enrollment_repo.create(&enrollment).map_err(|e| {
            if let EnrollmentRepoError::Database(ref msg) = e {
                if msg.contains("age_group") { return EnrollmentAppError::AgeGroupMismatch; }
                if msg.contains("capacity")  { return EnrollmentAppError::CourseFull; }
            }
            e.into()
        })?;
        log::info!("[enrollment] created: id={} student={} period={} price={}",
            enrollment.id(), input.student_id, input.course_period_id, course.price_cents());
        Ok(())
    }
}
