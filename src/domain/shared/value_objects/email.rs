#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    const MAX_LEN: usize = 40;
    const MIN_LEN: usize = 10;

    pub fn new(value: impl Into<String>) -> Result<Self, EmailError> {
        let s: String = value.into();
        if s.trim().is_empty()                { return Err(EmailError::Blank) }
        if s.chars().any(char::is_whitespace) { return Err(EmailError::Invalid) }
        if s.len() > Self::MAX_LEN            { return Err(EmailError::TooLong(Self::MAX_LEN)) }
        if s.len() < Self::MIN_LEN            { return Err(EmailError::TooShort(Self::MIN_LEN)) }
        let (a, b) = s.split_once('@').ok_or(EmailError::Invalid)?;
        if a.is_empty() || b.is_empty()       { return Err(EmailError::Invalid) }
        let (c, d) = b.rsplit_once('.').ok_or(EmailError::Invalid)?;
        if c.is_empty() || d.is_empty()       { return Err(EmailError::Invalid) }
        Ok(Self(s))
    }

    pub fn value(&self) -> &str { &self.0 }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("email cannot be blank")]
    Blank,
    #[error("invalid email address")]
    Invalid,
    #[error("email is too long (maximum {0} characters)")]
    TooLong(usize),
    #[error("email is too short (minimum {0} characters)")]
    TooShort(usize),
}
