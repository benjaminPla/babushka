pub mod repository;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::shared::value_objects::{
    email::Email,
    first_name::FirstName,
    last_name::LastName,
    phone::Phone,
};

pub struct Teacher {
    created_at: DateTime<Utc>,
    email:      Email,
    first_name: FirstName,
    id:         Uuid,
    last_name:  LastName,
    phone:      Phone,
    updated_at: DateTime<Utc>,
}

impl Teacher {
    pub fn new(
        email:      Email,
        first_name: FirstName,
        last_name:  LastName,
        phone:      Phone,
    ) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            email,
            first_name,
            id:         Uuid::new_v4(),
            last_name,
            phone,
            updated_at: now,
        }
    }

    pub fn reconstitute(
        created_at: DateTime<Utc>,
        email:      Email,
        first_name: FirstName,
        id:         Uuid,
        last_name:  LastName,
        phone:      Phone,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self { created_at, email, first_name, id, last_name, phone, updated_at }
    }

    // ── Mutations ────────────────────────────────────────────────────────────

    pub fn update(
        &mut self,
        email:      Email,
        first_name: FirstName,
        last_name:  LastName,
        phone:      Phone,
    ) {
        self.email      = email;
        self.first_name = first_name;
        self.last_name  = last_name;
        self.phone      = phone;
        self.updated_at = chrono::Utc::now();
    }

    // ── Getters ──────────────────────────────────────────────────────────────

    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn email(&self)      -> &Email        { &self.email }
    pub fn first_name(&self) -> &FirstName    { &self.first_name }
    pub fn id(&self)         -> Uuid          { self.id }
    pub fn last_name(&self)  -> &LastName     { &self.last_name }
    pub fn phone(&self)      -> &Phone        { &self.phone }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}
