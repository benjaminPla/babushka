use std::sync::Arc;

use crate::application::course::dto::CourseDto;
use crate::application::course::errors::CourseAppError;
use crate::domain::course::repository::CourseRepo;

pub struct CourseGetAllUseCase {
    course_repo: Arc<dyn CourseRepo>,
}

impl CourseGetAllUseCase {
    pub fn new(course_repo: Arc<dyn CourseRepo>) -> Self { Self { course_repo } }

    pub fn execute(&self) -> Result<Vec<CourseDto>, CourseAppError> {
        let courses = self.course_repo.get_all()?;
        Ok(courses.iter().map(CourseDto::from).collect())
    }
}
