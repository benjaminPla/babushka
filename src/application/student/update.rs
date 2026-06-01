use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::student::errors::StudentAppError,
    domain::{
        shared::value_objects::{
            email::Email,
            first_name::FirstName,
            last_name::LastName,
            phone::Phone,
        },
        student::{repository::StudentRepo, AgeGroup},
    },
};

pub struct StudentUpdateInput {
    pub id:         Uuid,
    pub age_group:  AgeGroup,
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub notes:      Option<String>,
    pub phone:      String,
}

pub struct StudentUpdateUseCase {
    student_repo: Arc<dyn StudentRepo>,
}

impl StudentUpdateUseCase {
    pub fn new(student_repo: Arc<dyn StudentRepo>) -> Self {
        Self { student_repo }
    }

    pub fn execute(&self, input: StudentUpdateInput) -> Result<(), StudentAppError> {
        let mut student = self.student_repo.get_by_id(input.id)?;
        let email       = Email::new(input.email)?;
        let first_name  = FirstName::new(input.first_name)?;
        let last_name   = LastName::new(input.last_name)?;
        let phone       = Phone::new(input.phone)?;
        student.update(input.age_group, email, first_name, last_name, input.notes, phone);
        self.student_repo.update(&student)?;
        log::info!("[student] updated: id={}", input.id);
        Ok(())
    }
}
