pub mod repository;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Overdue,
}

impl PaymentStatus {
    pub fn as_db_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Paid    => "paid",
            Self::Overdue => "overdue",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "paid"    => Some(Self::Paid),
            "overdue" => Some(Self::Overdue),
            _         => None,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Pending => "Pendiente",
            Self::Paid    => "Pagado",
            Self::Overdue => "Vencido",
        }
    }
}

pub struct Payment {
    id:            Uuid,
    enrollment_id: Uuid,
    student_name:  String,
    course_name:   String,
    amount_cents:  i32,
    due_date:      NaiveDate,
    paid_at:       Option<DateTime<Utc>>,
    status:        PaymentStatus,
    notes:         Option<String>,
    created_at:    DateTime<Utc>,
    updated_at:    DateTime<Utc>,
}

impl Payment {
    pub fn new(
        enrollment_id: Uuid,
        amount_cents:  i32,
        due_date:      NaiveDate,
        notes:         Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id:            Uuid::new_v4(),
            enrollment_id,
            student_name:  String::new(),
            course_name:   String::new(),
            amount_cents,
            due_date,
            paid_at:       None,
            status:        PaymentStatus::Pending,
            notes,
            created_at:    now,
            updated_at:    now,
        }
    }

    pub fn reconstitute(
        id:            Uuid,
        enrollment_id: Uuid,
        student_name:  String,
        course_name:   String,
        amount_cents:  i32,
        due_date:      NaiveDate,
        paid_at:       Option<DateTime<Utc>>,
        status:        PaymentStatus,
        notes:         Option<String>,
        created_at:    DateTime<Utc>,
        updated_at:    DateTime<Utc>,
    ) -> Self {
        Self { id, enrollment_id, student_name, course_name, amount_cents, due_date, paid_at, status, notes, created_at, updated_at }
    }

    pub fn id(&self)            -> Uuid                  { self.id }
    pub fn enrollment_id(&self) -> Uuid                  { self.enrollment_id }
    pub fn student_name(&self)  -> &str                  { &self.student_name }
    pub fn course_name(&self)   -> &str                  { &self.course_name }
    pub fn amount_cents(&self)  -> i32                   { self.amount_cents }
    pub fn due_date(&self)      -> NaiveDate             { self.due_date }
    pub fn paid_at(&self)       -> Option<DateTime<Utc>> { self.paid_at }
    pub fn status(&self)        -> &PaymentStatus        { &self.status }
    pub fn notes(&self)         -> Option<&str>          { self.notes.as_deref() }
    pub fn created_at(&self)    -> DateTime<Utc>         { self.created_at }
    pub fn updated_at(&self)    -> DateTime<Utc>         { self.updated_at }
}
