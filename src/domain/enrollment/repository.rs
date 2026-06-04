use uuid::Uuid;

use crate::domain::enrollment::Enrollment;

pub trait EnrollmentRepo: Send + Sync {
    fn create(&self, enrollment: &Enrollment)           -> Result<(), EnrollmentRepoError>;
    fn delete(&self, id: Uuid)                          -> Result<(), EnrollmentRepoError>;
    fn get_by_student(&self, student_id: Uuid)          -> Result<Vec<Enrollment>, EnrollmentRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EnrollmentRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("enrollment not found: {0}")]
    NotFound(Uuid),
}
