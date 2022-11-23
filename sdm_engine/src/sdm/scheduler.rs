use super::{Event, Process};
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicPtr, Ordering};
use uuid::Uuid;

static SCHEDULER_INSTANCE: AtomicPtr<Scheduler> = AtomicPtr::new(std::ptr::null_mut());

pub struct EventQueue(pub RefCell<Vec<(f32, Box<dyn Event>)>>);
pub struct ProcessQueue(pub RefCell<Vec<(f32, Box<dyn Process>)>>);

pub struct Scheduler {
    time: f32,                                                   // Simulation time
    event_queue: EventQueue,                                     // Future events
    process_queue: ProcessQueue,                                 // Future processes
    running_processes: RefCell<HashMap<Uuid, Box<dyn Process>>>, // Process to run every cicle
    process_finish_events: RefCell<Vec<(f32, Uuid)>>,            // Processes with on_end to run
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self);
        }

        SCHEDULER_INSTANCE.store(std::ptr::null_mut(), Ordering::Relaxed);
    }
}

impl Scheduler {
    pub fn new() -> Result<&'static mut Self> {
        if !Self::instanciated() {
            let instance = Box::new(Self {
                time: 0f32,
                event_queue: EventQueue(RefCell::new(vec![])),
                process_queue: ProcessQueue(RefCell::new(vec![])),
                running_processes: RefCell::new(HashMap::new()),
                process_finish_events: RefCell::new(vec![]),
            });

            SCHEDULER_INSTANCE.store(Box::into_raw(instance), Ordering::Relaxed);

            Self::instance()
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

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn instance() -> Result<&'static mut Self> {
        unsafe {
            if let Some(instance) = SCHEDULER_INSTANCE.load(Ordering::Relaxed).as_mut() {
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
            .sort_by(|a, b| a.0.total_cmp(&b.0).reverse())
    }

    fn sort_process_queue(&self) {
        self.process_queue
            .0
            .borrow_mut()
            .sort_by(|a, b| a.0.total_cmp(&b.0).reverse())
    }

    fn sort_process_finish_event_queue(&self) {
        self.process_finish_events
            .borrow_mut()
            .sort_by(|a, b| a.0.total_cmp(&b.0).reverse())
    }

    pub fn schedule_now(&self, event: Box<dyn Event>) {
        self.event_queue.0.borrow_mut().push((Self::time(), event));
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
        self.running_processes
            .borrow_mut()
            .insert(process.pid(), process);
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

    /// Check for processes that may be scheduled to start and start them
    fn check_process_queue(&self, future_time: &f32) {
        loop {
            if let Some((schedule_time, _)) = self.process_queue.0.borrow().last() {
                if schedule_time <= future_time {
                    let proc = self.process_queue.0.borrow_mut().pop().unwrap().1;
                    println!("{} - Starting process \"{}\"", schedule_time, proc.name());
                    self.running_processes.borrow_mut().insert(proc.pid(), proc);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn event_step(&self) -> bool {
        // Get first item from FEL
        let event = self.event_queue.0.borrow_mut().pop();

        if let Some(mut event) = event {
            // Set time to event time
            if Self::time() < event.0 {
                Self::instance().unwrap().set_time(event.0);
            } else if Self::time() > event.0 {
                // Sanity check
                panic!("Event time is in the past! Something has gone terribly wrong!")
            }

            // Dispatch event according to listener
            event.1.execute();

            // Execute processes and schedule on_end callbacks
            for (pid, proc) in self.running_processes.borrow_mut().iter_mut() {
                let duration = proc.start();
                self.process_finish_events
                    .borrow_mut()
                    .push((self.time + duration, pid.clone()));

                self.sort_process_finish_event_queue();
            }

            // Check for stop condition
            self.event_queue.0.borrow().is_empty()
        } else {
            true
        }
    }

    fn process_step(&self) {
        let (proc_time, proc_id) = self.process_finish_events.borrow_mut().pop().unwrap();

        // Set time to process time
        if Self::time() < proc_time {
            Self::instance().unwrap().set_time(proc_time);
        } else if Self::time() > proc_time {
            // Sanity check
            panic!("Event time is in the past! Something has gone terribly wrong!")
        }

        self.running_processes
            .borrow_mut()
            .get_mut(&proc_id)
            .expect("No process assossiated to PID")
            .end();
    }

    /// Simulates one step, returns whether stop condition is met
    /// A step can be either a process callback or an event from the FEL.
    /// If both are scheduled to the same time, the process callback takes precedence.
    pub fn simulate_one_step(&self) -> bool {
        let proc_time = match self.process_finish_events.borrow().last() {
            Some(proc_event) => Some(proc_event.0),
            None => None,
        };

        let event_time = match self.event_queue.0.borrow().last() {
            Some(event) => Some(event.0),
            None => None,
        };

        if let Some(proc_time) = proc_time {
            if let Some(event_time) = event_time {
                if proc_time <= event_time {
                    self.check_process_queue(&proc_time);
                    self.process_step();
                    false
                } else {
                    self.check_process_queue(&event_time);
                    self.event_step()
                }
            } else {
                self.check_process_queue(&proc_time);
                self.process_step();
                false
            }
        } else {
            self.check_process_queue(&self.event_queue.0.borrow().last().unwrap().0);
            self.event_step()
        }
    }

    pub fn simulate(&self) {
        while !self.simulate_one_step() {
            println!("--------------------------------------------------");
            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
            println!(
                "{} - Step complete. Events left in FEL: {}",
                self.time,
                self.event_queue.0.borrow().len()
            );
            println!("--------------------------------------------------");
        }
    }
}
