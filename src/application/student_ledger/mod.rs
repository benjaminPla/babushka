use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    enrollment::repository::EnrollmentRepo,
    payment::{PaymentStatus, repository::PaymentRepo},
};

#[derive(Clone, PartialEq)]
pub enum LedgerKind {
    Debt,
    Credit,
}

#[derive(Clone)]
pub struct LedgerEntry {
    pub id:              Uuid,
    pub kind:            LedgerKind,
    pub description:     String,
    pub amount_cents:    i32,
    pub running_balance: i32,
    pub date:            DateTime<Utc>,
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

    pub fn execute(&self, student_id: Uuid) -> Result<(Vec<LedgerEntry>, i32), StudentLedgerError> {
        let enrollments = self.enrollment_repo.get_by_student(student_id)
            .map_err(|_| StudentLedgerError::Database)?;
        let payments = self.payment_repo.get_by_student(student_id)
            .map_err(|_| StudentLedgerError::Database)?;

        let mut raw: Vec<(DateTime<Utc>, LedgerEntry)> = Vec::new();

        for e in &enrollments {
            raw.push((e.enrolled_at(), LedgerEntry {
                id:              e.id(),
                kind:            LedgerKind::Debt,
                description:     format!("Inscripción: {} — {}", e.course_name(), e.period_label()),
                amount_cents:    e.agreed_price_cents(),
                running_balance: 0,
                date:            e.enrolled_at(),
            }));
        }

        for p in &payments {
            if *p.status() == PaymentStatus::Paid {
                let method = match p.payment_method() {
                    Some("cash")     => "efectivo",
                    Some("transfer") => "transferencia",
                    Some("card")     => "tarjeta",
                    _                => "efectivo",
                };
                raw.push((p.paid_at().unwrap_or(p.created_at()), LedgerEntry {
                    id:              p.id(),
                    kind:            LedgerKind::Credit,
                    description:     format!("Pago ({}) — {} {}", method, p.course_name(), p.period_label()),
                    amount_cents:    p.amount_cents(),
                    running_balance: 0,
                    date:            p.paid_at().unwrap_or(p.created_at()),
                }));
            }
        }

        raw.sort_by_key(|(dt, _)| *dt);

        let mut balance = 0i32;
        let entries: Vec<LedgerEntry> = raw.into_iter().map(|(_, mut entry)| {
            match entry.kind {
                LedgerKind::Debt   => balance -= entry.amount_cents,
                LedgerKind::Credit => balance += entry.amount_cents,
            }
            entry.running_balance = balance;
            entry
        }).collect();

        Ok((entries, balance))
    }
}
