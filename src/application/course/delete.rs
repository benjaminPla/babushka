use std::sync::Arc;

use uuid::Uuid;

use crate::{application::course::errors::CourseAppError, domain::course::repository::CourseRepo};

pub struct CourseDeleteUseCase {
    course_repo: Arc<dyn CourseRepo>,
}

impl CourseDeleteUseCase {
    pub fn new(course_repo: Arc<dyn CourseRepo>) -> Self { Self { course_repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), CourseAppError> {
        self.course_repo.delete(id)?;
        log::info!("[course] deleted: id={}", id);
        Ok(())
    }
}
