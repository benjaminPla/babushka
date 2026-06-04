use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::course_period::CoursePeriod;

#[derive(Clone)]
pub struct CoursePeriodDto {
    pub id:         Uuid,
    pub label:      String,
    pub start_date: NaiveDate,
    pub end_date:   NaiveDate,
    pub enrolled:   i64,
}

impl From<&CoursePeriod> for CoursePeriodDto {
    fn from(p: &CoursePeriod) -> Self {
        Self {
            id:         p.id(),
            label:      p.label().to_owned(),
            start_date: p.start_date(),
            end_date:   p.end_date(),
            enrolled:   p.enrolled(),
        }
    }
}
