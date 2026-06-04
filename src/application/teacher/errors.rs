use crate::domain::{
    shared::value_objects::{
        email::EmailError,
        first_name::FirstNameError,
        last_name::LastNameError,
        notes::NotesError,
        phone::PhoneError,
    },
    teacher::repository::TeacherRepoError,
};

#[derive(Debug, thiserror::Error)]
pub enum TeacherAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("profesor no encontrado")]
    NotFound,
}

impl From<EmailError>     for TeacherAppError { fn from(e: EmailError)     -> Self { Self::Validation(e.to_string()) } }
impl From<FirstNameError> for TeacherAppError { fn from(e: FirstNameError) -> Self { Self::Validation(e.to_string()) } }
impl From<LastNameError>  for TeacherAppError { fn from(e: LastNameError)  -> Self { Self::Validation(e.to_string()) } }
impl From<NotesError>     for TeacherAppError { fn from(e: NotesError)     -> Self { Self::Validation(e.to_string()) } }
impl From<PhoneError>     for TeacherAppError { fn from(e: PhoneError)     -> Self { Self::Validation(e.to_string()) } }

impl From<TeacherRepoError> for TeacherAppError {
    fn from(e: TeacherRepoError) -> Self {
        match e {
            TeacherRepoError::Database(msg) => {
                log::error!("[teacher] repo error: {msg}");
                Self::Database
            }
            TeacherRepoError::NotFound(id) => {
                log::warn!("[teacher] not found: {id}");
                Self::NotFound
            }
        }
    }
}
