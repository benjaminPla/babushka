#[derive(Debug, Clone, Copy)]
pub struct CourseCapacity(i16);

impl CourseCapacity {
    pub fn new(value: i16) -> Result<Self, CourseCapacityError> {
        if value <= 0 { return Err(CourseCapacityError::NonPositive) }
        Ok(Self(value))
    }

    pub fn value(&self) -> i16 { self.0 }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CourseCapacityError {
    #[error("la capacidad debe ser mayor a 0")]
    NonPositive,
}
