use std::sync::Arc;

use crate::{
    application::teacher::{dto::TeacherDto, errors::TeacherAppError},
    domain::teacher::repository::TeacherRepo,
};

pub struct TeacherListUseCase {
    teacher_repo: Arc<dyn TeacherRepo>,
}

impl TeacherListUseCase {
    pub fn new(teacher_repo: Arc<dyn TeacherRepo>) -> Self {
        Self { teacher_repo }
    }

    pub fn execute(&self) -> Result<Vec<TeacherDto>, TeacherAppError> {
        let teachers = self.teacher_repo.list()?;
        Ok(teachers.iter().map(TeacherDto::from).collect())
    }
}
