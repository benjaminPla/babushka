use std::sync::Arc;

use crate::application::student::errors::StudentAppError;
use crate::domain::shared::value_objects::email::Email;
use crate::domain::shared::value_objects::first_name::FirstName;
use crate::domain::shared::value_objects::last_name::LastName;
use crate::domain::shared::value_objects::notes::Notes;
use crate::domain::shared::value_objects::phone::Phone;
use crate::domain::student::repository::StudentRepo;
use crate::domain::student::AgeGroup;
use crate::domain::student::Student;

pub struct StudentCreateInput {
    pub age_group:  AgeGroup,
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub notes:      Option<String>,
    pub phone:      String,
}

pub struct StudentCreateUseCase {
    student_repo: Arc<dyn StudentRepo>,
}

impl StudentCreateUseCase {
    pub fn new(student_repo: Arc<dyn StudentRepo>) -> Self {
        Self { student_repo }
    }

    pub fn execute(&self, input: StudentCreateInput) -> Result<(), StudentAppError> {
        let email      = Email::new(input.email)?;
        let first_name = FirstName::new(input.first_name)?;
        let last_name  = LastName::new(input.last_name)?;
        let notes      = input.notes.map(Notes::new).transpose()?;
        let phone      = Phone::new(input.phone)?;
        let student    = Student::new(input.age_group, email, first_name, last_name, notes, phone);
        self.student_repo.create(&student)?;
        log::info!("[student] created: id={}", student.id());
        Ok(())
    }
}
