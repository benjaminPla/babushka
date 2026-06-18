use std::collections::HashSet;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    enrollment::repository::EnrollmentRepo,
    payment::repository::PaymentRepo,
};

#[derive(Clone, PartialEq)]
pub enum LedgerKind {
    Pending,  // unsettled enrollment (no payment linked)
    Credit,   // payment
}

#[derive(Clone)]
pub struct LedgerEntry {
    pub id:             Uuid,
    pub kind:           LedgerKind,
    pub description:    String,
    pub payment_method: Option<String>,
    pub amount_cents:   Option<i32>,   // None for Pending
    pub course_id:      Option<Uuid>,  // Some for Pending (used for price lookup in UI)
    pub date:           DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum StudentLedgerError {
    #[error("error de base de datos")]
    Database,
}

pub struct StudentLedgerUseCase {
    enrollment_repo: Arc<dyn EnrollmentRepo>,
    payment_repo:    Arc<dyn PaymentRepo>,
}

impl StudentLedgerUseCase {
    pub fn new(enrollment_repo: Arc<dyn EnrollmentRepo>, payment_repo: Arc<dyn PaymentRepo>) -> Self {
        Self { enrollment_repo, payment_repo }
    }

    pub fn execute(&self, student_id: Uuid) -> Result<Vec<LedgerEntry>, StudentLedgerError> {
        let enrollments = self.enrollment_repo.get_by_student(student_id)
            .map_err(|_| StudentLedgerError::Database)?;
        let payments = self.payment_repo.get_by_student(student_id)
            .map_err(|_| StudentLedgerError::Database)?;

        let settled: HashSet<Uuid> = payments.iter()
            .filter_map(|p| p.enrollment_id())
            .collect();

        let mut raw: Vec<(DateTime<Utc>, LedgerEntry)> = Vec::new();

        for e in &enrollments {
            if !settled.contains(&e.id()) {
                raw.push((e.enrolled_at(), LedgerEntry {
                    id:             e.id(),
                    kind:           LedgerKind::Pending,
                    description:    format!("Inscripción: {} — {}", e.course_name(), e.period_label()),
                    payment_method: None,
                    amount_cents:   None,
                    course_id:      Some(e.course_id()),
                    date:           e.enrolled_at(),
                }));
            }
        }

        for p in &payments {
            let method = match p.payment_method() {
                "cash"     => "Efectivo",
                "transfer" => "Transferencia",
                "card"     => "Tarjeta",
                "discount" => "Descuento",
                other      => other,
            };
            raw.push((p.paid_at(), LedgerEntry {
                id:             p.id(),
                kind:           LedgerKind::Credit,
                description:    p.notes().map(str::to_string).unwrap_or_else(|| "—".into()),
                payment_method: Some(method.to_string()),
                amount_cents:   Some(p.amount_cents()),
                course_id:      None,
                date:           p.paid_at(),
            }));
        }

        raw.sort_by_key(|(dt, _)| *dt);
        let mut entries: Vec<LedgerEntry> = raw.into_iter().map(|(_, e)| e).collect();
        entries.reverse();

        Ok(entries)
    }
}
