use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Contact {
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub inserted_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Contact {
    pub fn new(
        uuid: String,
        name: String,
        email: String,
        phone: String,
        inserted_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Self {
        Contact {
            uuid,
            name,
            email,
            phone,
            inserted_at,
            updated_at,
        }
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }
}
