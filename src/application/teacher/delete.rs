use std::sync::Arc;

use uuid::Uuid;

use crate::application::teacher::errors::TeacherAppError;
use crate::domain::teacher::repository::TeacherRepo;

pub struct TeacherDeleteUseCase {
    teacher_repo: Arc<dyn TeacherRepo>,
}

impl TeacherDeleteUseCase {
    pub fn new(teacher_repo: Arc<dyn TeacherRepo>) -> Self {
        Self { teacher_repo }
    }

    pub fn execute(&self, id: Uuid) -> Result<(), TeacherAppError> {
        self.teacher_repo.delete(id)?;
        log::info!("[teacher] deleted: id={}", id);
        Ok(())
    }
}
