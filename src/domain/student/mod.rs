pub mod repository;

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub use crate::domain::shared::value_objects::age_group::AgeGroup;

use crate::domain::shared::value_objects::{
    email::Email,
    first_name::FirstName,
    last_name::LastName,
    notes::Notes,
    phone::Phone,
};

pub struct Student {
    age_group:  AgeGroup,
    created_at: DateTime<Utc>,
    email:      Option<Email>,
    first_name: FirstName,
    id:         Uuid,
    last_name:  LastName,
    notes:      Option<Notes>,
    phone:      Phone,
    updated_at: DateTime<Utc>,
}

impl Student {
    pub fn new(
        age_group:  AgeGroup,
        email:      Option<Email>,
        first_name: FirstName,
        last_name:  LastName,
        notes:      Option<Notes>,
        phone:      Phone,
    ) -> Self {
        let now = Utc::now();
        Self {
            age_group,
            created_at: now,
            email,
            first_name,
            id:         Uuid::new_v4(),
            last_name,
            notes,
            phone,
            updated_at: now,
        }
    }

    pub fn reconstitute(
        age_group:  AgeGroup,
        created_at: DateTime<Utc>,
        email:      Option<Email>,
        first_name: FirstName,
        id:         Uuid,
        last_name:  LastName,
        notes:      Option<Notes>,
        phone:      Phone,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            age_group,
            created_at,
            email,
            first_name,
            id,
            last_name,
            notes,
            phone,
            updated_at
        }
    }

    pub fn update(
        self,
        age_group:  AgeGroup,
        email:      Option<Email>,
        first_name: FirstName,
        last_name:  LastName,
        notes:      Option<Notes>,
        phone:      Phone,
    ) -> Self {
        Self {
            age_group,
            email,
            first_name,
            last_name,
            notes,
            phone,
            updated_at: Utc::now(),
            ..self
        }
    }

    // ── Getters ──────────────────────────────────────────────────────────────

    pub fn age_group(&self)  -> AgeGroup           { self.age_group }
    pub fn created_at(&self) -> DateTime<Utc>      { self.created_at }
    pub fn email(&self)      -> Option<&str>       { self.email.as_ref().map(Email::value) }
    pub fn first_name(&self) -> &str               { self.first_name.value() }
    pub fn id(&self)         -> Uuid               { self.id }
    pub fn last_name(&self)  -> &str               { self.last_name.value() }
    pub fn notes(&self)      -> Option<&str>       { self.notes.as_ref().map(Notes::value) }
    pub fn phone(&self)      -> &str               { self.phone.value() }
    pub fn updated_at(&self) -> DateTime<Utc>      { self.updated_at }
}
