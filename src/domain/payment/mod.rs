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
    id:             Uuid,
    student_id:     Uuid,
    amount_cents:   i32,
    due_date:       NaiveDate,
    paid_at:        Option<DateTime<Utc>>,
    payment_method: Option<String>,
    status:         PaymentStatus,
    notes:          Option<String>,
    created_at:     DateTime<Utc>,
}

impl Payment {
    pub fn new(student_id: Uuid, amount_cents: i32, due_date: NaiveDate, notes: Option<String>) -> Self {
        Self {
            id:             Uuid::new_v4(),
            student_id,
            amount_cents,
            due_date,
            paid_at:        None,
            payment_method: None,
            status:         PaymentStatus::Pending,
            notes,
            created_at:     Utc::now(),
        }
    }

    pub fn reconstitute(
        id:             Uuid,
        student_id:     Uuid,
        amount_cents:   i32,
        due_date:       NaiveDate,
        paid_at:        Option<DateTime<Utc>>,
        payment_method: Option<String>,
        status:         PaymentStatus,
        notes:          Option<String>,
        created_at:     DateTime<Utc>,
    ) -> Self {
        Self { id, student_id, amount_cents, due_date, paid_at, payment_method, status, notes, created_at }
    }

    pub fn id(&self)             -> Uuid                  { self.id }
    pub fn student_id(&self)     -> Uuid                  { self.student_id }
    pub fn amount_cents(&self)   -> i32                   { self.amount_cents }
    pub fn due_date(&self)       -> NaiveDate             { self.due_date }
    pub fn paid_at(&self)        -> Option<DateTime<Utc>> { self.paid_at }
    pub fn payment_method(&self) -> Option<&str>          { self.payment_method.as_deref() }
    pub fn status(&self)         -> &PaymentStatus        { &self.status }
    pub fn notes(&self)          -> Option<&str>          { self.notes.as_deref() }
    pub fn created_at(&self)     -> DateTime<Utc>         { self.created_at }
}
