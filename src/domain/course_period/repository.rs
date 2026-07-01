use uuid::Uuid;

use crate::domain::course_period::CoursePeriod;

pub trait CoursePeriodRepo: Send + Sync {
    fn create(&self, period: &CoursePeriod)       -> Result<(), CoursePeriodRepoError>;
    fn delete(&self, id: Uuid)                    -> Result<(), CoursePeriodRepoError>;
    fn get_by_course(&self, course_id: Uuid)      -> Result<Vec<CoursePeriod>, CoursePeriodRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CoursePeriodRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("{0}")]
    Duplicate(String),
    #[error("período no encontrado: {0}")]
    NotFound(Uuid),
}
