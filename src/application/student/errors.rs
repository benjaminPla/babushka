use crate::domain::{
    shared::value_objects::{
        email::EmailError,
        first_name::FirstNameError,
        last_name::LastNameError,
        phone::PhoneError,
    },
    student::repository::StudentRepoError,
};

#[derive(Debug, thiserror::Error)]
pub enum StudentAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("alumno no encontrado")]
    NotFound,
}

impl From<EmailError>     for StudentAppError { fn from(e: EmailError)     -> Self { Self::Validation(e.to_string()) } }
impl From<FirstNameError> for StudentAppError { fn from(e: FirstNameError) -> Self { Self::Validation(e.to_string()) } }
impl From<LastNameError>  for StudentAppError { fn from(e: LastNameError)  -> Self { Self::Validation(e.to_string()) } }
impl From<PhoneError>     for StudentAppError { fn from(e: PhoneError)     -> Self { Self::Validation(e.to_string()) } }

impl From<StudentRepoError> for StudentAppError {
    fn from(e: StudentRepoError) -> Self {
        match e {
            StudentRepoError::Database(msg) => {
                log::error!("[student] repo error: {msg}");
                Self::Database
            }
            StudentRepoError::NotFound(id) => {
                log::warn!("[student] not found: {id}");
                Self::NotFound
            }
        }
    }
}
