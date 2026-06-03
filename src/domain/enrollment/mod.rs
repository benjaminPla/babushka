pub mod repository;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum EffectiveStatus {
    Active,
    Dropped,
    Completed,
}

impl EffectiveStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Active    => "Activo",
            Self::Dropped   => "Baja",
            Self::Completed => "Completado",
        }
    }
}

pub struct Enrollment {
    id:               Uuid,
    student_id:       Uuid,
    student_name:     String,
    course_period_id: Uuid,
    period_label:     String,
    period_end_date:  NaiveDate,
    course_name:      String,
    dropped_at:       Option<DateTime<Utc>>,
    latest_payment:   Option<String>,
    enrolled_at:      DateTime<Utc>,
    updated_at:       DateTime<Utc>,
}

impl Enrollment {
    pub fn new(student_id: Uuid, course_period_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id:               Uuid::new_v4(),
            student_id,
            student_name:     String::new(),
            course_period_id,
            period_label:     String::new(),
            period_end_date:  NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(),
            course_name:      String::new(),
            dropped_at:       None,
            latest_payment:   None,
            enrolled_at:      now,
            updated_at:       now,
        }
    }

    pub fn reconstitute(
        id:               Uuid,
        student_id:       Uuid,
        student_name:     String,
        course_period_id: Uuid,
        period_label:     String,
        period_end_date:  NaiveDate,
        course_name:      String,
        dropped_at:       Option<DateTime<Utc>>,
        latest_payment:   Option<String>,
        enrolled_at:      DateTime<Utc>,
        updated_at:       DateTime<Utc>,
    ) -> Self {
        Self { id, student_id, student_name, course_period_id, period_label, period_end_date, course_name, dropped_at, latest_payment, enrolled_at, updated_at }
    }

    pub fn set_dropped(&mut self, dropped: bool) {
        self.dropped_at = if dropped { Some(Utc::now()) } else { None };
    }

    pub fn effective_status(&self) -> EffectiveStatus {
        if self.dropped_at.is_some() { return EffectiveStatus::Dropped; }
        if self.period_end_date < chrono::Local::now().date_naive() { return EffectiveStatus::Completed; }
        EffectiveStatus::Active
    }

    pub fn id(&self)                -> Uuid                   { self.id }
    pub fn student_id(&self)        -> Uuid                   { self.student_id }
    pub fn student_name(&self)      -> &str                   { &self.student_name }
    pub fn course_period_id(&self)  -> Uuid                   { self.course_period_id }
    pub fn period_label(&self)      -> &str                   { &self.period_label }
    pub fn period_end_date(&self)   -> NaiveDate              { self.period_end_date }
    pub fn course_name(&self)       -> &str                   { &self.course_name }
    pub fn dropped_at(&self)        -> Option<DateTime<Utc>>  { self.dropped_at }
    pub fn latest_payment(&self)    -> Option<&str>           { self.latest_payment.as_deref() }
    pub fn enrolled_at(&self)       -> DateTime<Utc>          { self.enrolled_at }
    pub fn updated_at(&self)        -> DateTime<Utc>          { self.updated_at }
}
