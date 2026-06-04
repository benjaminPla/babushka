pub mod repository;
pub mod value_objects;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    course::value_objects::{course_capacity::CourseCapacity, course_name::CourseName},
    shared::value_objects::{age_group::AgeGroup, cents::Cents, notes::Notes},
};

pub struct Course {
    age_group:         AgeGroup,
    capacity:          CourseCapacity,
    class_price_cents: Cents,
    created_at:        DateTime<Utc>,
    id:                Uuid,
    month_price_cents: Cents,
    name:              CourseName,
    notes:             Option<Notes>,
    teacher_id:        Uuid,
    updated_at:        DateTime<Utc>,
}

impl Course {
    pub fn new(
        age_group:         AgeGroup,
        capacity:          CourseCapacity,
        class_price_cents: Cents,
        month_price_cents: Cents,
        name:              CourseName,
        notes:             Option<Notes>,
        teacher_id:        Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            age_group,
            capacity,
            class_price_cents,
            created_at:        now,
            id:                Uuid::new_v4(),
            month_price_cents,
            name,
            notes,
            teacher_id,
            updated_at:        now,
        }
    }

    pub fn reconstitute(
        age_group:         AgeGroup,
        capacity:          CourseCapacity,
        class_price_cents: Cents,
        created_at:        DateTime<Utc>,
        id:                Uuid,
        month_price_cents: Cents,
        name:              CourseName,
        notes:             Option<Notes>,
        teacher_id:        Uuid,
        updated_at:        DateTime<Utc>,
    ) -> Self {
        Self {
            age_group,
            capacity,
            class_price_cents,
            created_at,
            id,
            month_price_cents,
            name,
            notes,
            teacher_id,
            updated_at,
        }
    }

    pub fn update(
        self,
        age_group:         AgeGroup,
        capacity:          CourseCapacity,
        class_price_cents: Cents,
        month_price_cents: Cents,
        name:              CourseName,
        notes:             Option<Notes>,
        teacher_id:        Uuid,
    ) -> Self {
        Self {
            age_group,
            capacity,
            class_price_cents,
            month_price_cents,
            name,
            notes,
            teacher_id,
            updated_at: Utc::now(),
            ..self
        }
    }

    // ── Getters ──────────────────────────────────────────────────────────────

    pub fn age_group(&self)         -> AgeGroup       { self.age_group }
    pub fn capacity(&self)          -> CourseCapacity { self.capacity }
    pub fn class_price_cents(&self) -> Cents          { self.class_price_cents }
    pub fn created_at(&self)        -> DateTime<Utc>  { self.created_at }
    pub fn id(&self)                -> Uuid           { self.id }
    pub fn month_price_cents(&self) -> Cents          { self.month_price_cents }
    pub fn name(&self)              -> &str           { self.name.value() }
    pub fn notes(&self)             -> Option<&str>   { self.notes.as_ref().map(Notes::value) }
    pub fn teacher_id(&self)        -> Uuid           { self.teacher_id }
    pub fn updated_at(&self)        -> DateTime<Utc>  { self.updated_at }
}
