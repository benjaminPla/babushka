use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::payment::{dto::PaymentDto, errors::PaymentAppError},
    domain::payment::repository::PaymentRepo,
};

pub struct PaymentGetByStudentUseCase {
    payment_repo: Arc<dyn PaymentRepo>,
}

impl PaymentGetByStudentUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepo>) -> Self { Self { payment_repo } }

    pub fn execute(&self, student_id: Uuid) -> Result<Vec<PaymentDto>, PaymentAppError> {
        let payments = self.payment_repo.get_by_student(student_id)?;
        Ok(payments.iter().map(PaymentDto::from).collect())
    }
}
