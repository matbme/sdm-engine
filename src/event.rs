use uuid::Uuid;

pub struct Event {
    name: String,
    id: Uuid,
}

impl Event {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: Uuid::new_v4(),
        }
    }
}
