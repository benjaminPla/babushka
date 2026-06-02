#[derive(Debug, Clone)]
pub struct FirstName(String);

impl FirstName {
    const MAX_CHARS: usize = 50;
    const MIN_CHARS: usize = 3;

    pub fn new(value: impl Into<String>) -> Result<Self, FirstNameError> {
        let s = value.into().trim().to_owned();
        if s.is_empty()              { return Err(FirstNameError::Empty) }
        if s.len() > Self::MAX_CHARS { return Err(FirstNameError::TooLong(Self::MAX_CHARS)) }
        if s.len() < Self::MIN_CHARS { return Err(FirstNameError::TooShort(Self::MIN_CHARS)) }
        Ok(Self(s))
    }

    pub fn value(&self) -> &str { &self.0 }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum FirstNameError {
    #[error("first_name cannot be empty")]
    Empty,
    #[error("first_name cannot be longer than {0} characters")]
    TooLong(usize),
    #[error("first_name cannot be shorter than {0} characters")]
    TooShort(usize),
}

