use super::Event;

pub struct Scheduler {
    time: f32,
    fel: Vec<(f32, Box<dyn Event>)>,
}

impl Scheduler {
    pub fn new(&self) -> Self {
        Self {
            time: 0f32,
            fel: vec![],
        }
    }

    pub fn time(&self) -> f32 {
        self.time
    }

}
