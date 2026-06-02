use uuid::Uuid;

use crate::domain::teacher::Teacher;

pub trait TeacherRepo: Send + Sync {
    fn create(&self, teacher: &Teacher) -> Result<(), TeacherRepoError>;
    fn delete(&self, id: Uuid)          -> Result<(), TeacherRepoError>;
    fn get_all(&self)                   -> Result<Vec<Teacher>, TeacherRepoError>;
    fn get_by_id(&self, id: Uuid)       -> Result<Teacher, TeacherRepoError>;
    fn update(&self, teacher: &Teacher) -> Result<(), TeacherRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum TeacherRepoError {
    #[error("database error")]
    Database(String),
    #[error("teacher not found")]
    NotFound(Uuid),
}
