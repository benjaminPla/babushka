use std::sync::Arc;

use uuid::Uuid;

use crate::application::course_period::dto::CoursePeriodDto;
use crate::application::course_period::errors::CoursePeriodAppError;
use crate::domain::course_period::repository::CoursePeriodRepo;

pub struct CoursePeriodGetByCourseUseCase {
    repo: Arc<dyn CoursePeriodRepo>,
}

impl CoursePeriodGetByCourseUseCase {
    pub fn new(repo: Arc<dyn CoursePeriodRepo>) -> Self { Self { repo } }

    pub fn execute(&self, course_id: Uuid) -> Result<Vec<CoursePeriodDto>, CoursePeriodAppError> {
        let periods = self.repo.get_by_course(course_id)?;
        Ok(periods.iter().map(CoursePeriodDto::from).collect())
    }
}
