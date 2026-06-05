pub const MAX_CAPACITY: i16 = 100;

#[derive(Debug, Clone, Copy)]
pub struct CourseCapacity(i16);

impl CourseCapacity {
    pub fn new(value: i16) -> Result<Self, CourseCapacityError> {
        if value <= 0           { return Err(CourseCapacityError::NonPositive) }
        if value > MAX_CAPACITY { return Err(CourseCapacityError::TooBig) }
        Ok(Self(value))
    }

    pub fn value(&self) -> i16 { self.0 }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CourseCapacityError {
    #[error("la capacidad debe ser mayor a 0")]
    NonPositive,
    #[error("la capacidad no puede superar {MAX_CAPACITY}")]
    TooBig,
}
