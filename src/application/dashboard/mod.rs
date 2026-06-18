use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use uuid::Uuid;

use crate::domain::course::repository::CourseRepo;
use crate::domain::enrollment::repository::EnrollmentRepo;
use crate::domain::student::{repository::StudentRepo, AgeGroup};
use crate::domain::teacher::repository::TeacherRepo;

pub struct DebtorRow {
    pub student_id:                 Uuid,
    pub full_name:                  String,
    pub enrollment_id:              Uuid,
    pub course_and_period:          String,
    pub pricing_type:               String,
    pub month_price_cash_cents:     i32,
    pub month_price_transfer_cents: i32,
    pub class_price_cash_cents:     i32,
    pub class_price_transfer_cents: i32,
}

pub struct DashboardData {
    pub debtors:            Vec<DebtorRow>,
    pub students_adult:     usize,
    pub students_minor:     usize,
    pub courses_adult:      usize,
    pub courses_minor:      usize,
    pub teachers_total:     usize,
    pub income_last_month:  i32,
    pub income_this_month:  i32,
    pub income_next_month:  i32,
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

        let course_map: HashMap<Uuid, _> = courses.iter().map(|c| (c.id(), c)).collect();

        let mut debtors: Vec<DebtorRow> = Vec::new();

        for s in &students {
            let enrollments = self.enrollment_repo.get_by_student(s.id())
                .map_err(|_| DashboardError::Database)?;

            for e in enrollments.iter().filter(|e| !e.is_paid()) {
                let Some(c) = course_map.get(&e.course_id()) else { continue };
                debtors.push(DebtorRow {
                    student_id:                 s.id(),
                    full_name:                  format!("{} {}", s.first_name(), s.last_name()),
                    enrollment_id:              e.id(),
                    course_and_period:          format!("{} — {}", e.course_name(), e.period_label()),
                    pricing_type:               e.pricing_type().to_owned(),
                    month_price_cash_cents:     c.month_price_cash_cents().value(),
                    month_price_transfer_cents: c.month_price_transfer_cents().value(),
                    class_price_cash_cents:     c.class_price_cash_cents().value(),
                    class_price_transfer_cents: c.class_price_transfer_cents().value(),
                });
            }
        }

        debtors.sort_by(|a, b| a.full_name.cmp(&b.full_name).then(a.course_and_period.cmp(&b.course_and_period)));

        let today    = Local::now();
        let (y, m)   = (today.year(), today.month());
        let this_m   = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let last_m   = if m == 1 { NaiveDate::from_ymd_opt(y - 1, 12, 1).unwrap() }
                       else      { NaiveDate::from_ymd_opt(y, m - 1, 1).unwrap() };
        let next_m   = if m == 12 { NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap() }
                       else       { NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap() };
        let after_next = if next_m.month() == 12 { NaiveDate::from_ymd_opt(next_m.year() + 1, 1, 1).unwrap() }
                         else                     { NaiveDate::from_ymd_opt(next_m.year(), next_m.month() + 1, 1).unwrap() };

        let to_utc = |d: NaiveDate| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap());

        let income_last_month = self.enrollment_repo
            .sum_paid_between(to_utc(last_m), to_utc(this_m))
            .map_err(|_| DashboardError::Database)?;
        let income_this_month = self.enrollment_repo
            .sum_paid_between(to_utc(this_m), to_utc(next_m))
            .map_err(|_| DashboardError::Database)?;
        let income_next_month = self.enrollment_repo
            .sum_expected_in_month(next_m, after_next)
            .map_err(|_| DashboardError::Database)?;

        Ok(DashboardData {
            debtors,
            students_adult,
            students_minor,
            courses_adult,
            courses_minor,
            teachers_total: teachers.len(),
            income_last_month,
            income_this_month,
            income_next_month,
        })
    }
}
