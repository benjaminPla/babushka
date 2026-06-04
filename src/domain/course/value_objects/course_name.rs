#[derive(Debug, Clone)]
pub struct CourseName(String);

impl CourseName {
    const MAX_CHARS: usize = 100;
    const MIN_CHARS: usize = 1;

    pub fn new(value: impl Into<String>) -> Result<Self, CourseNameError> {
        let s = value.into().trim().to_owned();
        if s.is_empty()              { return Err(CourseNameError::Empty) }
        if s.len() > Self::MAX_CHARS { return Err(CourseNameError::TooLong(Self::MAX_CHARS)) }
        if s.len() < Self::MIN_CHARS { return Err(CourseNameError::TooShort(Self::MIN_CHARS)) }
        Ok(Self(s))
    }

    pub fn value(&self) -> &str { &self.0 }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CourseNameError {
    #[error("el nombre del curse no puede estar vacío")]
    Empty,
    #[error("el nombre del curso no puede tener más de {0} caracteres")]
    TooLong(usize),
    #[error("el nombre del curso no puede tener menos de {0} caracteres")]
    TooShort(usize),
}

