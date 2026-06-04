use uuid::Uuid;

use crate::domain::course::Course;

pub trait CourseRepo: Send + Sync {
    fn create(&self, course: &Course)                                          -> Result<(), CourseRepoError>;
    fn delete(&self, id: Uuid)                                                 -> Result<(), CourseRepoError>;
    fn get_all(&self)                                                          -> Result<Vec<Course>, CourseRepoError>;
    fn get_by_id(&self, id: Uuid)                                              -> Result<Course, CourseRepoError>;
    fn update(&self, course: &Course)                                          -> Result<(), CourseRepoError>;
    fn count_enrollments(&self, course_id: Uuid)                               -> Result<i64, CourseRepoError>;
    fn has_age_group_conflict(&self, course_id: Uuid, age_group: &str)         -> Result<bool, CourseRepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CourseRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("course not found: {0}")]
    NotFound(Uuid),
}
