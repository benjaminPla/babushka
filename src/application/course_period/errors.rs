use crate::domain::course_period::repository::CoursePeriodRepoError;

#[derive(Debug, thiserror::Error)]
pub enum CoursePeriodAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("período no encontrado")]
    NotFound,
}

impl From<CoursePeriodRepoError> for CoursePeriodAppError {
    fn from(e: CoursePeriodRepoError) -> Self {
        match e {
            CoursePeriodRepoError::Duplicate(msg) => Self::Validation(msg),
            CoursePeriodRepoError::Database(msg) => {
                log::error!("[course_period] repo error: {msg}");
                Self::Database
            }
            CoursePeriodRepoError::NotFound(id) => {
                log::warn!("[course_period] not found: {id}");
                Self::NotFound
            }
        }
    }
}
