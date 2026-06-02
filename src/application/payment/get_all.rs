use std::sync::Arc;

use crate::{
    application::payment::{dto::PaymentDto, errors::PaymentAppError},
    domain::payment::repository::PaymentRepo,
};

pub struct PaymentGetAllUseCase {
    payment_repo: Arc<dyn PaymentRepo>,
}

impl PaymentGetAllUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepo>) -> Self { Self { payment_repo } }

    pub fn execute(&self) -> Result<Vec<PaymentDto>, PaymentAppError> {
        let payments = self.payment_repo.get_all()?;
        Ok(payments.iter().map(PaymentDto::from).collect())
    }
}
