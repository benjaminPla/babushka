pub mod repository;

use chrono::NaiveDate;
use uuid::Uuid;

pub struct CoursePeriod {
    id:         Uuid,
    course_id:  Uuid,
    label:      String,
    start_date: NaiveDate,
    end_date:   NaiveDate,
    enrolled:   i64,
}

impl CoursePeriod {
    pub fn new(course_id: Uuid, label: String, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            course_id,
            label,
            start_date,
            end_date,
            enrolled: 0,
        }
    }

    pub fn reconstitute(
        id:         Uuid,
        course_id:  Uuid,
        label:      String,
        start_date: NaiveDate,
        end_date:   NaiveDate,
        enrolled:   i64,
    ) -> Self {
        Self { id, course_id, label, start_date, end_date, enrolled }
    }

    pub fn id(&self)         -> Uuid      { self.id }
    pub fn course_id(&self)  -> Uuid      { self.course_id }
    pub fn label(&self)      -> &str      { &self.label }
    pub fn start_date(&self) -> NaiveDate { self.start_date }
    pub fn end_date(&self)   -> NaiveDate { self.end_date }
    pub fn enrolled(&self)   -> i64       { self.enrolled }
}
