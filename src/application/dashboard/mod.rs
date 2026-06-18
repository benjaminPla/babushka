use std::sync::Arc;

use uuid::Uuid;

use crate::domain::course::repository::CourseRepo;
use crate::domain::enrollment::repository::EnrollmentRepo;
use crate::domain::student::{repository::StudentRepo, AgeGroup};
use crate::domain::teacher::repository::TeacherRepo;

pub struct DebtorRow {
    pub id:        Uuid,
    pub full_name: String,
    pub pending:   String,
}

pub struct DashboardData {
    pub debtors:         Vec<DebtorRow>,
    pub students_adult:  usize,
    pub students_minor:  usize,
    pub courses_adult:   usize,
    pub courses_minor:   usize,
    pub teachers_total:  usize,
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("error de base de datos")]
    Database,
}

pub struct DashboardUseCase {
    student_repo:    Arc<dyn StudentRepo>,
    enrollment_repo: Arc<dyn EnrollmentRepo>,
    course_repo:     Arc<dyn CourseRepo>,
    teacher_repo:    Arc<dyn TeacherRepo>,
}

impl DashboardUseCase {
    pub fn new(
        student_repo:    Arc<dyn StudentRepo>,
        enrollment_repo: Arc<dyn EnrollmentRepo>,
        course_repo:     Arc<dyn CourseRepo>,
        teacher_repo:    Arc<dyn TeacherRepo>,
    ) -> Self {
        Self { student_repo, enrollment_repo, course_repo, teacher_repo }
    }

    pub fn load(&self) -> Result<DashboardData, DashboardError> {
        let students = self.student_repo.get_all().map_err(|_| DashboardError::Database)?;
        let courses  = self.course_repo.get_all().map_err(|_| DashboardError::Database)?;
        let teachers = self.teacher_repo.get_all().map_err(|_| DashboardError::Database)?;

        let students_adult = students.iter().filter(|s| s.age_group() == AgeGroup::Adult).count();
        let students_minor = students.iter().filter(|s| s.age_group() == AgeGroup::Minor).count();
        let courses_adult  = courses.iter().filter(|c| c.age_group() == AgeGroup::Adult).count();
        let courses_minor  = courses.iter().filter(|c| c.age_group() == AgeGroup::Minor).count();

        let mut debtors: Vec<DebtorRow> = students
            .iter()
            .filter_map(|s| {
                let enrollments = self.enrollment_repo.get_by_student(s.id()).ok()?;
                let unpaid: Vec<_> = enrollments.iter().filter(|e| !e.is_paid()).collect();
                if unpaid.is_empty() { return None; }
                let pending = unpaid.iter()
                    .map(|e| format!("{} — {}", e.course_name(), e.period_label()))
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(DebtorRow {
                    id:        s.id(),
                    full_name: format!("{} {}", s.first_name(), s.last_name()),
                    pending,
                })
            })
            .collect();

        debtors.sort_by(|a, b| a.full_name.cmp(&b.full_name));

        Ok(DashboardData {
            debtors,
            students_adult,
            students_minor,
            courses_adult,
            courses_minor,
            teachers_total: teachers.len(),
        })
    }
}
