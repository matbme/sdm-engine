use super::{Event, Process};
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

static SCHEDULER_TIME: AtomicPtr<f32> = AtomicPtr::new(std::ptr::null_mut());
static SCHEDULER_INSTANCIATED: AtomicBool = AtomicBool::new(false);

pub struct EventQueue(pub RefCell<Vec<(f32, Box<dyn Event>)>>);
pub struct ProcessQueue(pub RefCell<Vec<(f32, Box<dyn Process>)>>);

pub struct Scheduler {
    time: f32,
    event_queue: EventQueue,
    process_queue: ProcessQueue,
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        SCHEDULER_TIME.store(std::ptr::null_mut(), Ordering::Relaxed);
        SCHEDULER_INSTANCIATED.store(false, Ordering::Relaxed);
    }
}

impl Scheduler {
    pub fn new() -> Result<Self> {
        if !Self::instanciated() {
            let mut instance = Self {
                time: 0f32,
                event_queue: EventQueue(RefCell::new(vec![])),
                process_queue: ProcessQueue(RefCell::new(vec![])),
            };

            SCHEDULER_TIME.store(std::ptr::addr_of_mut!(instance.time), Ordering::Relaxed);
            SCHEDULER_INSTANCIATED.store(true, Ordering::Relaxed);

            Ok(instance)
        } else {
            Err(anyhow!("There is already another instance of Scheduler, please drop it before creating a new scheduler instance"))
        }
    }

    pub fn time() -> f32 {
        unsafe {
            SCHEDULER_TIME
                .load(Ordering::Relaxed)
                .as_ref()
                .expect("No scheduler has been instanciated")
                .clone()
        }
    }

    pub fn instanciated() -> bool {
        SCHEDULER_INSTANCIATED.load(Ordering::Relaxed)
    }

    pub fn schedule_now(&self, event: Box<dyn Event>) {
        self.event_queue
            .0
            .borrow_mut()
            .insert(0, (Self::time(), event));
    }

    pub fn schedule_in(&self, event: Box<dyn Event>, time_to_event: f32) {
        self.event_queue
            .0
            .borrow_mut()
            .push((Self::time() + time_to_event, event));
    }

    pub fn schedule_at(&self, event: Box<dyn Event>, schedule_time: f32) {
        self.event_queue.0.borrow_mut().push((schedule_time, event));
    }

    pub fn start_process_now(&self, process: Box<dyn Process>) {
        self.process_queue
            .0
            .borrow_mut()
            .insert(0, (Self::time(), process));
    }

    pub fn start_process_in(&self, process: Box<dyn Process>, time_to_process: f32) {
        self.process_queue
            .0
            .borrow_mut()
            .push((Self::time() + time_to_process, process));
    }

    pub fn start_process_at(&self, process: Box<dyn Process>, schedule_time: f32) {
        self.process_queue
            .0
            .borrow_mut()
            .push((schedule_time, process));
    }

    // pub fn wait_for(&self, time: f32) {
    //
    // }

    pub fn simulate(&self) {}
}
