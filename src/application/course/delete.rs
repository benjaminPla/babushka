use std::sync::Arc;

use uuid::Uuid;

use crate::application::course::errors::CourseAppError;
use crate::domain::course::repository::CourseRepo;

pub struct CourseDeleteUseCase {
    course_repo: Arc<dyn CourseRepo>,
}

impl CourseDeleteUseCase {
    pub fn new(course_repo: Arc<dyn CourseRepo>) -> Self { Self { course_repo } }

    pub fn execute(&self, id: Uuid) -> Result<(), CourseAppError> {
        let enrolled = self.course_repo.count_enrollments(id)?;
        if enrolled > 0 {
            return Err(CourseAppError::Validation(
                format!("no se puede eliminar el curso: tiene {} alumnos inscriptos", enrolled)
            ));
        }
        self.course_repo.delete(id)?;
        log::info!("[course] deleted: id={}", id);
        Ok(())
    }
}
