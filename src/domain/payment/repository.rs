use uuid::Uuid;

use crate::domain::payment::Payment;

pub trait PaymentRepo: Send + Sync {
    fn create(&self, payment: &Payment)        -> Result<(), PaymentRepoError>;
    fn delete(&self, id: Uuid)                 -> Result<(), PaymentRepoError>;
    fn get_by_student(&self, student_id: Uuid) -> Result<Vec<Payment>, PaymentRepoError>;
    fn mark_paid(&self, id: Uuid)              -> Result<(), PaymentRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("payment not found: {0}")]
    NotFound(Uuid),
}
