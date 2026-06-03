use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::course::errors::CourseAppError,
    domain::{course::repository::CourseRepo, shared::value_objects::age_group::AgeGroup},
};

pub struct CourseUpdateInput {
    pub id:                Uuid,
    pub teacher_id:        Uuid,
    pub name:              String,
    pub age_group:         AgeGroup,
    pub capacity:          i16,
    pub price_cents:       i32,
    pub class_price_cents: i32,
    pub notes:             Option<String>,
}

pub struct CourseUpdateUseCase {
    course_repo: Arc<dyn CourseRepo>,
}

impl CourseUpdateUseCase {
    pub fn new(course_repo: Arc<dyn CourseRepo>) -> Self { Self { course_repo } }

    pub fn execute(&self, input: CourseUpdateInput) -> Result<(), CourseAppError> {
        if input.name.trim().is_empty()    { return Err(CourseAppError::Validation("nombre requerido".into())); }
        if input.name.len() > 100          { return Err(CourseAppError::Validation("nombre demasiado largo".into())); }
        if input.capacity <= 0             { return Err(CourseAppError::Validation("capacidad debe ser mayor a 0".into())); }
        if input.price_cents <= 0          { return Err(CourseAppError::Validation("precio debe ser mayor a 0".into())); }
        if input.class_price_cents <= 0    { return Err(CourseAppError::Validation("precio por clase debe ser mayor a 0".into())); }
        let mut course = self.course_repo.get_by_id(input.id)?;
        course.update(input.teacher_id, input.name, input.age_group, input.capacity, input.price_cents, input.class_price_cents, input.notes);
        self.course_repo.update(&course)?;
        log::info!("[course] updated: id={}", input.id);
        Ok(())
    }
}
