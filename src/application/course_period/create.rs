use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::application::course_period::errors::CoursePeriodAppError;
use crate::domain::course_period::repository::CoursePeriodRepo;
use crate::domain::course_period::CoursePeriod;

pub struct CoursePeriodCreateInput {
    pub course_id:  Uuid,
    pub start_date: NaiveDate,
    pub end_date:   NaiveDate,
}

pub struct CoursePeriodCreateUseCase {
    repo: Arc<dyn CoursePeriodRepo>,
}

impl CoursePeriodCreateUseCase {
    pub fn new(repo: Arc<dyn CoursePeriodRepo>) -> Self { Self { repo } }

    pub fn execute(&self, input: CoursePeriodCreateInput) -> Result<(), CoursePeriodAppError> {
        if input.end_date <= input.start_date {
            return Err(CoursePeriodAppError::Validation("la fecha de fin debe ser posterior a la de inicio".into()));
        }
        let period = CoursePeriod::new(input.course_id, input.start_date, input.end_date);
        self.repo.create(&period)?;
        log::info!("[course_period] created: id={} course={}", period.id(), input.course_id);
        Ok(())
    }
}
