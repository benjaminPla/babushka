use crate::domain::payment::repository::PaymentRepoError;

#[derive(Debug, thiserror::Error)]
pub enum PaymentAppError {
    #[error("{0}")]
    Validation(String),
    #[error("error de base de datos")]
    Database,
    #[error("pago no encontrado")]
    NotFound,
}

impl From<PaymentRepoError> for PaymentAppError {
    fn from(e: PaymentRepoError) -> Self {
        match e {
            PaymentRepoError::Database(msg) => {
                log::error!("[payment] repo error: {msg}");
                Self::Database
            }
            PaymentRepoError::NotFound(id) => {
                log::warn!("[payment] not found: {id}");
                Self::NotFound
            }
        }
    }
}
