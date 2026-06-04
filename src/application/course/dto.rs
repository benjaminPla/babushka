use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{course::Course, shared::value_objects::age_group::AgeGroup};

#[derive(Clone)]
pub struct CourseDto {
    pub id:                Uuid,
    pub teacher_id:        Uuid,
    pub name:              String,
    pub age_group:         AgeGroup,
    pub capacity:          i16,
    pub month_price_cents: i32,
    pub class_price_cents: i32,
    pub notes:             Option<String>,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

impl From<&Course> for CourseDto {
    fn from(c: &Course) -> Self {
        Self {
            id:                c.id(),
            teacher_id:        c.teacher_id(),
            name:              c.name().to_owned(),
            age_group:         c.age_group(),
            capacity:          c.capacity().value(),
            month_price_cents: c.month_price_cents().value(),
            class_price_cents: c.class_price_cents().value(),
            notes:             c.notes().map(str::to_owned),
            created_at:        c.created_at(),
            updated_at:        c.updated_at(),
        }
    }
}
