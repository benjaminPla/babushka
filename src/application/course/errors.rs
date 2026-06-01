use crate::domain::course::repository::CourseRepoError;

#[derive(Debug, thiserror::Error)]
pub enum CourseAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("curso no encontrado")]
    NotFound,
}

impl From<CourseRepoError> for CourseAppError {
    fn from(e: CourseRepoError) -> Self {
        match e {
            CourseRepoError::Database(msg) => {
                log::error!("[course] repo error: {msg}");
                Self::Database
            }
            CourseRepoError::NotFound(id) => {
                log::warn!("[course] not found: {id}");
                Self::NotFound
            }
        }
    }
}
