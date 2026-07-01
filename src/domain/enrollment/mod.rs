pub mod repository;
pub mod value_objects;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::enrollment::value_objects::{
    payment_method::PaymentMethod,
    pricing_type::PricingType,
};

pub struct Enrollment {
    id:                Uuid,
    student_id:        Uuid,
    course_period_id:  Uuid,
    course_id:         Uuid,
    period_label:      String,
    course_name:       String,
    student_name:      String,
    pricing_type:      PricingType,
    paid_amount_cents: Option<i32>,
    payment_method:    Option<PaymentMethod>,
    paid_at:           Option<DateTime<Utc>>,
    payment_notes:     Option<String>,
}

impl Enrollment {
    pub fn new(student_id: Uuid, course_period_id: Uuid, pricing_type: PricingType) -> Self {
        Self {
            id:                Uuid::new_v4(),
            student_id,
            course_period_id,
            course_id:         Uuid::nil(),
            period_label:      String::new(),
            course_name:       String::new(),
            student_name:      String::new(),
            pricing_type,
            paid_amount_cents: None,
            payment_method:    None,
            paid_at:           None,
            payment_notes:     None,
        }
    }

    pub fn reconstitute(
        id:                Uuid,
        student_id:        Uuid,
        course_period_id:  Uuid,
        course_id:         Uuid,
        period_label:      String,
        course_name:       String,
        student_name:      String,
        pricing_type:      PricingType,
        paid_amount_cents: Option<i32>,
        payment_method:    Option<PaymentMethod>,
        paid_at:           Option<DateTime<Utc>>,
        payment_notes:     Option<String>,
    ) -> Self {
        Self {
            id, student_id, course_period_id, course_id, period_label, course_name,
            student_name, pricing_type, paid_amount_cents, payment_method,
            paid_at, payment_notes,
        }
    }

    pub fn id(&self)                -> Uuid                  { self.id }
    pub fn student_id(&self)        -> Uuid                  { self.student_id }
    pub fn course_period_id(&self)  -> Uuid                  { self.course_period_id }
    pub fn course_id(&self)         -> Uuid                  { self.course_id }
    pub fn period_label(&self)      -> &str                  { &self.period_label }
    pub fn course_name(&self)       -> &str                  { &self.course_name }
    pub fn student_name(&self)      -> &str                  { &self.student_name }
    pub fn pricing_type(&self)      -> &str                  { self.pricing_type.value() }
    pub fn is_monthly(&self)        -> bool                  { self.pricing_type.is_monthly() }
    pub fn paid_amount_cents(&self) -> Option<i32>           { self.paid_amount_cents }
    pub fn payment_method(&self)    -> Option<&str>          { self.payment_method.as_ref().map(|m| m.value()) }
    pub fn paid_at(&self)           -> Option<DateTime<Utc>> { self.paid_at }
    pub fn payment_notes(&self)     -> Option<&str>          { self.payment_notes.as_deref() }
    pub fn is_paid(&self)           -> bool                  { self.paid_at.is_some() }
}
