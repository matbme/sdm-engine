use uuid::Uuid;

pub struct Process {
    name: String,
    pid: Uuid,
    duration: f32,
    active: bool,
}

impl Process {
    pub fn new(name: &str, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            pid: Uuid::new_v4(),
            duration,
            active: false,
        }
    }

    pub fn duration(&self) -> &f32 {
        &self.duration
    }

    pub fn change_duration(&mut self, duration: f32) {
        self.duration = duration;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn toggle_activate(&mut self) {
        self.active = !self.active
    }
}
