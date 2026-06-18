pub mod repository;

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Enrollment {
    id:               Uuid,
    student_id:       Uuid,
    course_period_id: Uuid,
    course_id:        Uuid,
    period_label:     String,
    course_name:      String,
    enrolled_at:      DateTime<Utc>,
}

impl Enrollment {
    pub fn new(student_id: Uuid, course_period_id: Uuid) -> Self {
        Self {
            id:               Uuid::new_v4(),
            student_id,
            course_period_id,
            course_id:        Uuid::nil(),
            period_label:     String::new(),
            course_name:      String::new(),
            enrolled_at:      Utc::now(),
        }
    }

    pub fn reconstitute(
        id:               Uuid,
        student_id:       Uuid,
        course_period_id: Uuid,
        course_id:        Uuid,
        period_label:     String,
        course_name:      String,
        enrolled_at:      DateTime<Utc>,
    ) -> Self {
        Self { id, student_id, course_period_id, course_id, period_label, course_name, enrolled_at }
    }

    pub fn id(&self)               -> Uuid          { self.id }
    pub fn student_id(&self)       -> Uuid          { self.student_id }
    pub fn course_period_id(&self) -> Uuid          { self.course_period_id }
    pub fn course_id(&self)        -> Uuid          { self.course_id }
    pub fn period_label(&self)     -> &str          { &self.period_label }
    pub fn course_name(&self)      -> &str          { &self.course_name }
    pub fn enrolled_at(&self)      -> DateTime<Utc> { self.enrolled_at }
}
