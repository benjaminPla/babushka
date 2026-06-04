use crate::domain::{
    course::{
        repository::CourseRepoError,
        value_objects::{
            course_capacity::CourseCapacityError,
            course_name::CourseNameError,
        },
    },
    shared::value_objects::{cents::CentsError, notes::NotesError},
};

#[derive(Debug, thiserror::Error)]
pub enum CourseAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("curso no encontrado")]
    NotFound,
}

impl From<CentsError>        for CourseAppError { fn from(e: CentsError)        -> Self { Self::Validation(e.to_string()) } }
impl From<CourseCapacityError> for CourseAppError { fn from(e: CourseCapacityError) -> Self { Self::Validation(e.to_string()) } }
impl From<CourseNameError>   for CourseAppError { fn from(e: CourseNameError)   -> Self { Self::Validation(e.to_string()) } }
impl From<NotesError>        for CourseAppError { fn from(e: NotesError)        -> Self { Self::Validation(e.to_string()) } }

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
