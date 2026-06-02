use uuid::Uuid;

use crate::domain::enrollment::Enrollment;

pub trait EnrollmentRepo: Send + Sync {
    fn create(&self, enrollment: &Enrollment)   -> Result<(), EnrollmentRepoError>;
    fn delete(&self, id: Uuid)                  -> Result<(), EnrollmentRepoError>;
    fn get_all(&self)                           -> Result<Vec<Enrollment>, EnrollmentRepoError>;
    fn get_by_course(&self, course_id: Uuid)    -> Result<Vec<Enrollment>, EnrollmentRepoError>;
    fn get_by_id(&self, id: Uuid)               -> Result<Enrollment, EnrollmentRepoError>;
    fn count_active(&self, course_id: Uuid)     -> Result<i64, EnrollmentRepoError>;
    fn update(&self, enrollment: &Enrollment)   -> Result<(), EnrollmentRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EnrollmentRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("enrollment not found: {0}")]
    NotFound(Uuid),
}
