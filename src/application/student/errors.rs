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
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("not found")]
    NotFound,
}

impl From<EmailError>     for StudentAppError { fn from(e: EmailError)     -> Self { Self::Validation(e.to_string()) } }
impl From<FirstNameError> for StudentAppError { fn from(e: FirstNameError) -> Self { Self::Validation(e.to_string()) } }
impl From<LastNameError>  for StudentAppError { fn from(e: LastNameError)  -> Self { Self::Validation(e.to_string()) } }
impl From<PhoneError>     for StudentAppError { fn from(e: PhoneError)     -> Self { Self::Validation(e.to_string()) } }

impl From<StudentRepoError> for StudentAppError {
    fn from(e: StudentRepoError) -> Self {
        match e {
            StudentRepoError::Database(msg) => Self::Database(msg),
            StudentRepoError::NotFound(_)   => Self::NotFound,
        }
    }
}
