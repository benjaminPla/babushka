use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::enrollment::{Enrollment, EnrollmentStatus};

#[derive(Clone)]
pub struct EnrollmentDto {
    pub id:             Uuid,
    pub student_id:     Uuid,
    pub student_name:   String,
    pub course_id:      Uuid,
    pub course_name:    String,
    pub status:         EnrollmentStatus,
    pub latest_payment: Option<String>,
    pub enrolled_at:    DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

impl From<&Enrollment> for EnrollmentDto {
    fn from(e: &Enrollment) -> Self {
        Self {
            id:             e.id(),
            student_id:     e.student_id(),
            student_name:   e.student_name().to_owned(),
            course_id:      e.course_id(),
            course_name:    e.course_name().to_owned(),
            status:         e.status().clone(),
            latest_payment: e.latest_payment().map(str::to_owned),
            enrolled_at:    e.enrolled_at(),
            updated_at:     e.updated_at(),
        }
    }
}
