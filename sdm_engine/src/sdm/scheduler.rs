use super::{Event, Process};
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};

static SCHEDULER_INSTANCE: AtomicPtr<Scheduler> = AtomicPtr::new(std::ptr::null_mut());

pub struct EventQueue(pub RefCell<Vec<(f32, Box<dyn Event>)>>);
pub struct ProcessQueue(pub RefCell<Vec<(f32, Box<dyn Process>)>>);

pub struct Scheduler {
    time: f32,
    event_queue: EventQueue,
    started_processes: RefCell<Vec<Box<dyn Process>>>,
    process_queue: ProcessQueue,
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        SCHEDULER_INSTANCE.store(std::ptr::null_mut(), Ordering::Relaxed);
    }
}

impl Scheduler {
    pub fn new() -> Result<&'static Self> {
        if !Self::instanciated() {
            // FIXME: Needs to be created as static
            let mut instance = Self {
                time: 0f32,
                event_queue: EventQueue(RefCell::new(vec![])),
                started_processes: RefCell::new(vec![]),
                process_queue: ProcessQueue(RefCell::new(vec![])),
            };

            SCHEDULER_INSTANCE.store(std::ptr::addr_of_mut!(instance), Ordering::Relaxed);

            Ok(&instance)
        } else {
            Err(anyhow!("There is already another instance of Scheduler, please drop it before creating a new scheduler instance"))
        }
    }

    pub fn time() -> f32 {
        unsafe {
            SCHEDULER_INSTANCE
                .load(Ordering::Relaxed)
                .as_ref()
                .expect("No scheduler has been instanciated")
                .time
        }
    }

    pub fn instance() -> Result<&'static Self> {
        unsafe {
            if let Some(instance) = SCHEDULER_INSTANCE.load(Ordering::Relaxed).as_ref() {
                Ok(instance)
            } else {
                Err(anyhow!("No scheduler has been instanciated"))
            }
        }
    }

    pub fn instanciated() -> bool {
        !SCHEDULER_INSTANCE.load(Ordering::Relaxed).is_null()
    }

    fn sort_event_queue(&self) {
        self.event_queue
            .0
            .borrow_mut()
            .sort_by(|a, b| a.0.total_cmp(&b.0))
    }

    fn sort_process_queue(&self) {
        self.process_queue
            .0
            .borrow_mut()
            .sort_by(|a, b| a.0.total_cmp(&b.0))
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

        self.sort_event_queue();
    }

    pub fn schedule_at(&self, event: Box<dyn Event>, schedule_time: f32) {
        self.event_queue.0.borrow_mut().push((schedule_time, event));

        self.sort_event_queue();
    }

    pub fn start_process_now(&self, process: Box<dyn Process>) {
        self.started_processes.borrow_mut().push(process);
    }

    pub fn start_process_in(&self, process: Box<dyn Process>, time_to_process: f32) {
        self.process_queue
            .0
            .borrow_mut()
            .push((Self::time() + time_to_process, process));

        self.sort_process_queue()
    }

    pub fn start_process_at(&self, process: Box<dyn Process>, schedule_time: f32) {
        self.process_queue
            .0
            .borrow_mut()
            .push((schedule_time, process));

        self.sort_process_queue()
    }

    // pub fn wait_for(&self, time: f32) {
    //
    // }

    pub fn simulate_one_step(&self) {
        // Get first item from FEL

        // Set time to event time

        // Dispatch event according to listener

        // Execute processes(?) for entities

        // Check for stop condition
    }
}
