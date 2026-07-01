use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::enrollment::{
    value_objects::payment_method::PaymentMethod,
    Enrollment,
};

pub trait EnrollmentRepo: Send + Sync {
    fn create(&self, enrollment: &Enrollment)                        -> Result<(), EnrollmentRepoError>;
    fn delete(&self, id: Uuid)                                       -> Result<(), EnrollmentRepoError>;
    fn get_by_student(&self, student_id: Uuid)                       -> Result<Vec<Enrollment>, EnrollmentRepoError>;
    fn get_by_course(&self, course_id: Uuid)                         -> Result<Vec<Enrollment>, EnrollmentRepoError>;
    fn sum_paid_between(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<i32, EnrollmentRepoError>;
    fn sum_expected_in_month(&self, from: NaiveDate, to: NaiveDate)    -> Result<i32, EnrollmentRepoError>;
    fn pay(&self, id: Uuid, amount_cents: i32, method: PaymentMethod, paid_at: DateTime<Utc>, notes: Option<String>) -> Result<(), EnrollmentRepoError>;
    fn delete_payment(&self, id: Uuid)                               -> Result<(), EnrollmentRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EnrollmentRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("{0}")]
    Duplicate(String),
    #[error("enrollment not found: {0}")]
    NotFound(Uuid),
}
