use std::sync::Arc;

use crate::{
    application::teacher::errors::TeacherAppError,
    domain::{
        shared::value_objects::{
            email::Email,
            first_name::FirstName,
            last_name::LastName,
            phone::Phone,
        },
        teacher::{repository::TeacherRepo, Teacher},
    },
};

pub struct TeacherCreateInput {
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub phone:      String,
}

pub struct TeacherCreateUseCase {
    teacher_repo: Arc<dyn TeacherRepo>,
}

impl TeacherCreateUseCase {
    pub fn new(teacher_repo: Arc<dyn TeacherRepo>) -> Self {
        Self { teacher_repo }
    }

    pub fn execute(&self, input: TeacherCreateInput) -> Result<(), TeacherAppError> {
        let email      = Email::new(input.email)?;
        let first_name = FirstName::new(input.first_name)?;
        let last_name  = LastName::new(input.last_name)?;
        let phone      = Phone::new(input.phone)?;
        let teacher    = Teacher::new(email, first_name, last_name, phone);
        self.teacher_repo.create(&teacher)?;
        Ok(())
    }
}
