use uuid::Uuid;

use crate::domain::student::Student;

pub trait StudentRepo: Send + Sync {
    fn create(&self, student: &Student)  -> Result<(), StudentRepoError>;
    fn delete(&self, id: Uuid)           -> Result<(), StudentRepoError>;
    fn get_all(&self)                    -> Result<Vec<Student>, StudentRepoError>;
    fn get_by_id(&self, id: Uuid)        -> Result<Student, StudentRepoError>;
    fn update(&self, student: &Student)  -> Result<(), StudentRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StudentRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("student not found: {0}")]
    NotFound(Uuid),
}
