use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::teacher::{dto::TeacherDto, errors::TeacherAppError},
    domain::teacher::repository::TeacherRepo,
};

pub struct TeacherGetByIdUseCase {
    teacher_repo: Arc<dyn TeacherRepo>,
}

impl TeacherGetByIdUseCase {
    pub fn new(teacher_repo: Arc<dyn TeacherRepo>) -> Self {
        Self { teacher_repo }
    }

    pub fn execute(&self, id: Uuid) -> Result<TeacherDto, TeacherAppError> {
        let teacher = self.teacher_repo.get_by_id(id)?;
        Ok(TeacherDto::from(&teacher))
    }
}
