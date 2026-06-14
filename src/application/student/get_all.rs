use std::sync::Arc;

use crate::application::student::dto::StudentDto;
use crate::application::student::errors::StudentAppError;
use crate::domain::student::repository::StudentRepo;

pub struct StudentGetAllUseCase {
    student_repo: Arc<dyn StudentRepo>,
}

impl StudentGetAllUseCase {
    pub fn new(student_repo: Arc<dyn StudentRepo>) -> Self {
        Self { student_repo }
    }

    pub fn execute(&self) -> Result<Vec<StudentDto>, StudentAppError> {
        let students = self.student_repo.get_all()?;
        Ok(students.iter().map(StudentDto::from).collect())
    }
}
