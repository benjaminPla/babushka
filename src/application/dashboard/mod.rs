use std::sync::Arc;

use uuid::Uuid;

use crate::application::student_ledger::StudentLedgerUseCase;
use crate::domain::enrollment::repository::EnrollmentRepo;
use crate::domain::payment::repository::PaymentRepo;
use crate::domain::student::repository::StudentRepo;

pub struct DebtorRow {
    pub id:            Uuid,
    pub full_name:     String,
    pub balance_cents: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("error de base de datos")]
    Database,
}

pub struct DashboardUseCase {
    student_repo:    Arc<dyn StudentRepo>,
    enrollment_repo: Arc<dyn EnrollmentRepo>,
    payment_repo:    Arc<dyn PaymentRepo>,
}

impl DashboardUseCase {
    pub fn new(
        student_repo:    Arc<dyn StudentRepo>,
        enrollment_repo: Arc<dyn EnrollmentRepo>,
        payment_repo:    Arc<dyn PaymentRepo>,
    ) -> Self {
        Self { student_repo, enrollment_repo, payment_repo }
    }

    pub fn debtors(&self) -> Result<Vec<DebtorRow>, DashboardError> {
        let students = self.student_repo
            .get_all()
            .map_err(|_| DashboardError::Database)?;

        let ledger_uc = StudentLedgerUseCase::new(
            Arc::clone(&self.enrollment_repo),
            Arc::clone(&self.payment_repo),
        );

        let mut rows: Vec<DebtorRow> = students
            .iter()
            .filter_map(|s| {
                let (_, balance) = ledger_uc.execute(s.id()).ok()?;
                (balance < 0).then(|| DebtorRow {
                    id:            s.id(),
                    full_name:     format!("{} {}", s.first_name(), s.last_name()),
                    balance_cents: balance,
                })
            })
            .collect();

        rows.sort_by_key(|r| r.balance_cents);
        Ok(rows)
    }
}
