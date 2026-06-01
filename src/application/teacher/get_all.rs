use std::sync::Arc;

use crate::{
    application::teacher::{dto::TeacherDto, errors::TeacherAppError},
    domain::teacher::repository::TeacherRepo,
};

pub struct TeacherGetAllUseCase {
    teacher_repo: Arc<dyn TeacherRepo>,
}

impl TeacherGetAllUseCase {
    pub fn new(teacher_repo: Arc<dyn TeacherRepo>) -> Self {
        Self { teacher_repo }
    }

    pub fn execute(&self) -> Result<Vec<TeacherDto>, TeacherAppError> {
        let teachers = self.teacher_repo.get_all()?;
        Ok(teachers.iter().map(TeacherDto::from).collect())
    }
}
