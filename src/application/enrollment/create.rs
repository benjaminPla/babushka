use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::enrollment::errors::EnrollmentAppError,
    domain::{
        course::repository::CourseRepo,
        enrollment::{repository::EnrollmentRepo, Enrollment},
        student::repository::StudentRepo,
    },
};

pub struct EnrollmentCreateInput {
    pub student_id: Uuid,
    pub course_id:  Uuid,
}

pub struct EnrollmentCreateUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
    course_repo:     Arc<dyn CourseRepo>,
    student_repo:    Arc<dyn StudentRepo>,
}

impl EnrollmentCreateUseCase {
    pub fn new(
        enrollment_repo: Arc<dyn EnrollmentRepo>,
        course_repo:     Arc<dyn CourseRepo>,
        student_repo:    Arc<dyn StudentRepo>,
    ) -> Self {
        Self { enrollment_repo, course_repo, student_repo }
    }

    pub fn execute(&self, input: EnrollmentCreateInput) -> Result<(), EnrollmentAppError> {
        let course = self.course_repo.get_by_id(input.course_id).map_err(|e| {
            log::error!("[enrollment] course lookup failed: {e}");
            EnrollmentAppError::NotFound
        })?;
        let student = self.student_repo.get_by_id(input.student_id).map_err(|e| {
            log::error!("[enrollment] student lookup failed: {e}");
            EnrollmentAppError::NotFound
        })?;

        if student.age_group() != course.age_group() {
            log::warn!(
                "[enrollment] age_group mismatch: student={} course={}",
                input.student_id, input.course_id
            );
            return Err(EnrollmentAppError::AgeGroupMismatch);
        }

        let active = self.enrollment_repo.count_active(input.course_id).map_err(|e| {
            log::error!("[enrollment] count_active failed: {e}");
            EnrollmentAppError::Database
        })?;
        if active >= course.capacity() as i64 {
            log::warn!("[enrollment] course full: {} ({}/{})", input.course_id, active, course.capacity());
            return Err(EnrollmentAppError::CourseFull);
        }

        let enrollment = Enrollment::new(input.student_id, input.course_id);
        self.enrollment_repo.create(&enrollment)?;
        log::info!("[enrollment] created: id={} student={} course={}", enrollment.id(), input.student_id, input.course_id);
        Ok(())
    }
}
