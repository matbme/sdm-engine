pub struct Scheduler {
    time: f32,
}

impl Scheduler {
    pub fn new(&self) -> Self {
        Self { time: 0f32 }
    }

    pub fn time(&self) -> f32 {
        self.time
    }
}
