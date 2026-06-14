use std::sync::Arc;

use uuid::Uuid;

use crate::application::student::errors::StudentAppError;
use crate::domain::student::repository::StudentRepo;

pub struct StudentDeleteUseCase {
    student_repo: Arc<dyn StudentRepo>,
}

impl StudentDeleteUseCase {
    pub fn new(student_repo: Arc<dyn StudentRepo>) -> Self {
        Self { student_repo }
    }

    pub fn execute(&self, id: Uuid) -> Result<(), StudentAppError> {
        self.student_repo.delete(id)?;
        log::info!("[student] deleted: id={}", id);
        Ok(())
    }
}
