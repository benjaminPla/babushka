use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::enrollment::{EffectiveStatus, Enrollment};

#[derive(Clone)]
pub struct EnrollmentDto {
    pub id:               Uuid,
    pub student_id:       Uuid,
    pub student_name:     String,
    pub course_period_id: Uuid,
    pub period_label:     String,
    pub period_end_date:  NaiveDate,
    pub course_name:      String,
    pub dropped_at:       Option<DateTime<Utc>>,
    pub latest_payment:   Option<String>,
    pub enrolled_at:      DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}

impl EnrollmentDto {
    pub fn effective_status(&self) -> EffectiveStatus {
        if self.dropped_at.is_some() { return EffectiveStatus::Dropped; }
        if self.period_end_date < chrono::Local::now().date_naive() { return EffectiveStatus::Completed; }
        EffectiveStatus::Active
    }
}

impl From<&Enrollment> for EnrollmentDto {
    fn from(e: &Enrollment) -> Self {
        Self {
            id:               e.id(),
            student_id:       e.student_id(),
            student_name:     e.student_name().to_owned(),
            course_period_id: e.course_period_id(),
            period_label:     e.period_label().to_owned(),
            period_end_date:  e.period_end_date(),
            course_name:      e.course_name().to_owned(),
            dropped_at:       e.dropped_at(),
            latest_payment:   e.latest_payment().map(str::to_owned),
            enrolled_at:      e.enrolled_at(),
            updated_at:       e.updated_at(),
        }
    }
}
