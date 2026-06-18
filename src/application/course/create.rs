use std::sync::Arc;

use uuid::Uuid;

use crate::application::course::errors::CourseAppError;
use crate::domain::course::repository::CourseRepo;
use crate::domain::course::value_objects::course_capacity::CourseCapacity;
use crate::domain::course::value_objects::course_name::CourseName;
use crate::domain::course::Course;
use crate::domain::shared::value_objects::age_group::AgeGroup;
use crate::domain::shared::value_objects::cents::Cents;
use crate::domain::shared::value_objects::notes::Notes;

pub struct CourseCreateInput {
    pub age_group:                  AgeGroup,
    pub capacity:                   i16,
    pub class_price_cash_cents:     i32,
    pub class_price_transfer_cents: i32,
    pub month_price_cash_cents:     i32,
    pub month_price_transfer_cents: i32,
    pub name:                       String,
    pub notes:                      Option<String>,
    pub teacher_id:                 Uuid,
}

pub struct CourseCreateUseCase {
    course_repo: Arc<dyn CourseRepo>,
}

impl CourseCreateUseCase {
    pub fn new(course_repo: Arc<dyn CourseRepo>) -> Self { Self { course_repo } }

    pub fn execute(&self, input: CourseCreateInput) -> Result<(), CourseAppError> {
        let capacity                   = CourseCapacity::new(input.capacity)?;
        let class_price_cash_cents     = Cents::new(input.class_price_cash_cents)?;
        let class_price_transfer_cents = Cents::new(input.class_price_transfer_cents)?;
        let month_price_cash_cents     = Cents::new(input.month_price_cash_cents)?;
        let month_price_transfer_cents = Cents::new(input.month_price_transfer_cents)?;
        let name                       = CourseName::new(input.name)?;
        let notes                      = input.notes.map(Notes::new).transpose()?;
        let course = Course::new(
            input.age_group, capacity,
            class_price_cash_cents, class_price_transfer_cents,
            month_price_cash_cents, month_price_transfer_cents,
            name, notes, input.teacher_id,
        );
        self.course_repo.create(&course)?;
        log::info!("[course] created: id={}", course.id());
        Ok(())
    }
}
