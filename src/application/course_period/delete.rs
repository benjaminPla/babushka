use std::sync::Arc;

use uuid::Uuid;

use crate::application::course_period::errors::CoursePeriodAppError;
use crate::domain::course_period::repository::CoursePeriodRepo;

pub struct CoursePeriodDeleteUseCase {
    repo: Arc<dyn CoursePeriodRepo>,
}

impl CoursePeriodDeleteUseCase {
    pub fn new(repo: Arc<dyn CoursePeriodRepo>) -> Self { Self { repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), CoursePeriodAppError> {
        self.repo.delete(id)?;
        log::info!("[course_period] deleted: id={}", id);
        Ok(())
    }
}
