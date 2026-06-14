use std::sync::Arc;

use uuid::Uuid;

use crate::application::course::errors::CourseAppError;
use crate::domain::course::repository::CourseRepo;
use crate::domain::course::value_objects::course_capacity::CourseCapacity;
use crate::domain::course::value_objects::course_name::CourseName;
use crate::domain::shared::value_objects::age_group::AgeGroup;
use crate::domain::shared::value_objects::cents::Cents;
use crate::domain::shared::value_objects::notes::Notes;

pub struct CourseUpdateInput {
    pub id:                Uuid,
    pub age_group:         AgeGroup,
    pub capacity:          i16,
    pub class_price_cents: i32,
    pub month_price_cents: i32,
    pub name:              String,
    pub notes:             Option<String>,
    pub teacher_id:        Uuid,
}

pub struct CourseUpdateUseCase {
    course_repo: Arc<dyn CourseRepo>,
}

impl CourseUpdateUseCase {
    pub fn new(course_repo: Arc<dyn CourseRepo>) -> Self { Self { course_repo } }

    pub fn execute(&self, input: CourseUpdateInput) -> Result<(), CourseAppError> {
        let course            = self.course_repo.get_by_id(input.id)?;
        let capacity          = CourseCapacity::new(input.capacity)?;
        let class_price_cents = Cents::new(input.class_price_cents)?;
        let month_price_cents = Cents::new(input.month_price_cents)?;
        let name              = CourseName::new(input.name)?;
        let notes             = input.notes.map(Notes::new).transpose()?;

        // Prevent lowering capacity below current enrollment count
        let enrolled = self.course_repo.count_enrollments(input.id)?;
        if (capacity.value() as i64) < enrolled {
            return Err(CourseAppError::Validation(
                format!("no se puede reducir la capacidad: hay {} alumnos inscriptos", enrolled)
            ));
        }

        // Prevent age_group change when enrolled students don't match
        if input.age_group != course.age_group() {
            let conflict = self.course_repo.has_age_group_conflict(input.id, input.age_group.as_db_str())?;
            if conflict {
                return Err(CourseAppError::Validation(
                    "no se puede cambiar el grupo de edad: hay alumnos inscriptos del grupo opuesto".into()
                ));
            }
        }

        let updated = course.update(input.age_group, capacity, class_price_cents, month_price_cents, name, notes, input.teacher_id);
        self.course_repo.update(&updated)?;
        log::info!("[course] updated: id={}", input.id);
        Ok(())
    }
}
