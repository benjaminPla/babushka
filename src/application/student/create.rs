use std::sync::Arc;

use crate::{
    application::student::errors::StudentAppError,
    domain::{
        shared::value_objects::{
            email::Email,
            first_name::FirstName,
            last_name::LastName,
            phone::Phone,
        },
        student::{repository::StudentRepo, AgeGroup, Student},
    },
};

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
        let phone      = Phone::new(input.phone)?;
        let student    = Student::new(input.age_group, email, first_name, last_name, input.notes, phone);
        self.student_repo.create(&student)?;
        log::info!("[student] created: id={}", student.id());
        Ok(())
    }
}
