#[derive(Debug, Clone)]
pub struct LastName(String);

impl LastName {
    const MAX_CHARS: usize = 50;
    const MIN_CHARS: usize = 3;

    pub fn new(value: impl Into<String>) -> Result<Self, LastNameError> {
        let s = value.into();
        if s.trim().is_empty()       { return Err(LastNameError::Empty) }
        if s.len() > Self::MAX_CHARS { return Err(LastNameError::TooLong(Self::MAX_CHARS)) }
        if s.len() < Self::MIN_CHARS { return Err(LastNameError::TooShort(Self::MIN_CHARS)) }
        Ok(Self(s))
    }

    pub fn value(&self) -> &str { &self.0 }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum LastNameError {
    #[error("last_name cannot be empty")]
    Empty,
    #[error("last_name cannot be longer than {0} characters")]
    TooLong(usize),
    #[error("last_name cannot be shorter than {0} characters")]
    TooShort(usize),
}
