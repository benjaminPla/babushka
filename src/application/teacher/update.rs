use std::sync::Arc;

use uuid::Uuid;

use crate::application::teacher::errors::TeacherAppError;
use crate::domain::shared::value_objects::email::Email;
use crate::domain::shared::value_objects::first_name::FirstName;
use crate::domain::shared::value_objects::last_name::LastName;
use crate::domain::shared::value_objects::notes::Notes;
use crate::domain::shared::value_objects::phone::Phone;
use crate::domain::teacher::repository::TeacherRepo;

pub struct TeacherUpdateInput {
    pub id:         Uuid,
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub notes:      Option<String>,
    pub phone:      String,
}

pub struct TeacherUpdateUseCase {
    teacher_repo: Arc<dyn TeacherRepo>,
}

impl TeacherUpdateUseCase {
    pub fn new(teacher_repo: Arc<dyn TeacherRepo>) -> Self {
        Self { teacher_repo }
    }

    pub fn execute(&self, input: TeacherUpdateInput) -> Result<(), TeacherAppError> {
        let teacher    = self.teacher_repo.get_by_id(input.id)?;
        let email      = Email::new(input.email)?;
        let first_name = FirstName::new(input.first_name)?;
        let last_name  = LastName::new(input.last_name)?;
        let notes      = input.notes.map(Notes::new).transpose()?;
        let phone      = Phone::new(input.phone)?;
        let updated    = teacher.update(email, first_name, last_name, notes, phone);
        self.teacher_repo.update(&updated)?;
        log::info!("[teacher] updated: id={}", input.id);
        Ok(())
    }
}
