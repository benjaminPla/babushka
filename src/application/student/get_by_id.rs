use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::student::{dto::StudentDto, errors::StudentAppError},
    domain::student::repository::StudentRepo,
};

pub struct StudentGetByIdUseCase {
    student_repo: Arc<dyn StudentRepo>,
}

impl StudentGetByIdUseCase {
    pub fn new(student_repo: Arc<dyn StudentRepo>) -> Self {
        Self { student_repo }
    }

    pub fn execute(&self, id: Uuid) -> Result<StudentDto, StudentAppError> {
        let student = self.student_repo.get_by_id(id)?;
        Ok(StudentDto::from(&student))
    }
}
