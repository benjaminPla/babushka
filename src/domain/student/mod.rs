pub mod repository;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::shared::value_objects::{
    email::Email,
    first_name::FirstName,
    last_name::LastName,
    phone::Phone,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AgeGroup {
    #[default]
    Adult,
    Minor,
}

impl AgeGroup {
    pub fn as_db_str(&self) -> &str {
        match self {
            Self::Adult => "adult",
            Self::Minor => "minor",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "adult" => Some(Self::Adult),
            "minor" => Some(Self::Minor),
            _       => None,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Adult => "Adulto",
            Self::Minor => "Menor",
        }
    }
}

pub struct Student {
    age_group:  AgeGroup,
    created_at: DateTime<Utc>,
    email:      Email,
    first_name: FirstName,
    id:         Uuid,
    last_name:  LastName,
    notes:      Option<String>,
    phone:      Phone,
    updated_at: DateTime<Utc>,
}

impl Student {
    pub fn new(
        age_group:  AgeGroup,
        email:      Email,
        first_name: FirstName,
        last_name:  LastName,
        notes:      Option<String>,
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
        email:      Email,
        first_name: FirstName,
        id:         Uuid,
        last_name:  LastName,
        notes:      Option<String>,
        phone:      Phone,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self { age_group, created_at, email, first_name, id, last_name, notes, phone, updated_at }
    }

    // ── Mutations ────────────────────────────────────────────────────────────

    pub fn update(
        &mut self,
        age_group:  AgeGroup,
        email:      Email,
        first_name: FirstName,
        last_name:  LastName,
        notes:      Option<String>,
        phone:      Phone,
    ) {
        self.age_group  = age_group;
        self.email      = email;
        self.first_name = first_name;
        self.last_name  = last_name;
        self.notes      = notes;
        self.phone      = phone;
        self.updated_at = Utc::now();
    }

    // ── Getters ──────────────────────────────────────────────────────────────

    pub fn age_group(&self)  -> &AgeGroup       { &self.age_group }
    pub fn created_at(&self) -> DateTime<Utc>   { self.created_at }
    pub fn email(&self)      -> &Email          { &self.email }
    pub fn first_name(&self) -> &FirstName      { &self.first_name }
    pub fn id(&self)         -> Uuid            { self.id }
    pub fn last_name(&self)  -> &LastName       { &self.last_name }
    pub fn notes(&self)      -> Option<&str>    { self.notes.as_deref() }
    pub fn phone(&self)      -> &Phone          { &self.phone }
    pub fn updated_at(&self) -> DateTime<Utc>   { self.updated_at }
}
