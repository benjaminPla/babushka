use crate::domain::enrollment::repository::EnrollmentRepoError;

#[derive(Debug, thiserror::Error)]
pub enum EnrollmentAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("inscripción no encontrada")]
    NotFound,
    #[error("el grupo de edad del alumno no coincide con el del curso")]
    AgeGroupMismatch,
    #[error("el curso ha alcanzado su capacidad máxima")]
    CourseFull,
}

impl From<EnrollmentRepoError> for EnrollmentAppError {
    fn from(e: EnrollmentRepoError) -> Self {
        match e {
            EnrollmentRepoError::Database(msg) => {
                log::error!("[enrollment] repo error: {msg}");
                Self::Database
            }
            EnrollmentRepoError::NotFound(id) => {
                log::warn!("[enrollment] not found: {id}");
                Self::NotFound
            }
        }
    }
}
