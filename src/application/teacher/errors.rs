use crate::domain::{
    shared::value_objects::{
        email::EmailError,
        first_name::FirstNameError,
        last_name::LastNameError,
        phone::PhoneError,
    },
    teacher::repository::TeacherRepoError,
};

#[derive(Debug, thiserror::Error)]
pub enum TeacherAppError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("not found")]
    NotFound,
}

impl From<EmailError>     for TeacherAppError { fn from(e: EmailError)     -> Self { Self::Validation(e.to_string()) } }
impl From<FirstNameError> for TeacherAppError { fn from(e: FirstNameError) -> Self { Self::Validation(e.to_string()) } }
impl From<LastNameError>  for TeacherAppError { fn from(e: LastNameError)  -> Self { Self::Validation(e.to_string()) } }
impl From<PhoneError>     for TeacherAppError { fn from(e: PhoneError)     -> Self { Self::Validation(e.to_string()) } }

impl From<TeacherRepoError> for TeacherAppError {
    fn from(e: TeacherRepoError) -> Self {
        match e {
            TeacherRepoError::Database(msg) => Self::Database(msg),
            TeacherRepoError::NotFound(_)   => Self::NotFound,
        }
    }
}
