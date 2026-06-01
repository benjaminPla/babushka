#[derive(Debug, Clone)]
pub struct Phone(String);

impl Phone {
    pub fn new(raw: String) -> Result<Self, PhoneError> {
        let normalized: String = raw
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect();

        if !normalized.starts_with('+') {
            return Err(PhoneError::Invalid);
        }

        let digits = &normalized[1..];
        if !digits.chars().all(|c| c.is_ascii_digit()) || !(8..=15).contains(&digits.len()) {
            return Err(PhoneError::Invalid);
        }

        Ok(Self(normalized))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("Invalid phone: must be E.164 format (e.g. +33612345678), 8–15 digits after '+'")]
    Invalid,
}
