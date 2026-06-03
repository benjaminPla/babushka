pub mod repository;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum EnrollmentStatus {
    #[default]
    Active,
    Dropped,
    Completed,
}

impl EnrollmentStatus {
    pub fn as_db_str(&self) -> &str {
        match self {
            Self::Active    => "active",
            Self::Dropped   => "dropped",
            Self::Completed => "completed",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "active"    => Some(Self::Active),
            "dropped"   => Some(Self::Dropped),
            "completed" => Some(Self::Completed),
            _           => None,
        }
    }

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
    course_name:      String,
    status:           EnrollmentStatus,
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
            course_name:      String::new(),
            status:           EnrollmentStatus::Active,
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
        course_name:      String,
        status:           EnrollmentStatus,
        latest_payment:   Option<String>,
        enrolled_at:      DateTime<Utc>,
        updated_at:       DateTime<Utc>,
    ) -> Self {
        Self { id, student_id, student_name, course_period_id, period_label, course_name, status, latest_payment, enrolled_at, updated_at }
    }

    pub fn set_status(&mut self, status: EnrollmentStatus) { self.status = status; }

    pub fn id(&self)                 -> Uuid              { self.id }
    pub fn student_id(&self)         -> Uuid              { self.student_id }
    pub fn student_name(&self)       -> &str              { &self.student_name }
    pub fn course_period_id(&self)   -> Uuid              { self.course_period_id }
    pub fn period_label(&self)       -> &str              { &self.period_label }
    pub fn course_name(&self)        -> &str              { &self.course_name }
    pub fn status(&self)             -> &EnrollmentStatus { &self.status }
    pub fn latest_payment(&self)     -> Option<&str>      { self.latest_payment.as_deref() }
    pub fn enrolled_at(&self)        -> DateTime<Utc>     { self.enrolled_at }
    pub fn updated_at(&self)         -> DateTime<Utc>     { self.updated_at }
}
