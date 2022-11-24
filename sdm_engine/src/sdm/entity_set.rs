use anyhow::Result;
use uuid::Uuid;

use super::Entity;

#[derive(Clone, Copy)]
pub enum EntitySetMode {
    FIFO,
    LIFO,
    PRIORITY,
}

impl Default for EntitySetMode {
    fn default() -> Self {
        Self::FIFO
    }
}

pub trait EntitySet {
    fn mode(&self) -> EntitySetMode;

    fn sort_container(&self);

    fn push(&self, entity: Box<dyn Entity>);

    fn pop(&self) -> Option<Box<dyn Entity>>;

    fn remove(&self, id: Uuid) -> Option<Box<dyn Entity>>;

    fn is_empty(&self) -> bool;

    fn is_full(&self) -> bool;

    fn apply_for_id<F: Fn(&dyn Entity) -> ()>(&self, id: Uuid, func: F) -> Result<()>
    where
        Self: Sized;

    fn size(&self) -> usize;

    fn average_size(&self) -> f32;

    fn max_size(&self) -> Option<usize>;

    fn update_analytics(&self);

    fn average_time_in_set(&self) -> f32;

    fn max_time_in_set(&self) -> f32;
}

#[macro_export]
macro_rules! EntitySetWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ; ) => {
        #[derive(Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            mode: sdm_engine::sdm::EntitySetMode,
            max_size: Option<usize>,
            average_size_sum: std::cell::RefCell<u32>,
            max_time_in_set: std::cell::RefCell<f32>,
            container: std::cell::RefCell<Vec<(f32, Box<dyn Entity>)>>,
            $($(
                $varname: $type,
            )*)?
        }

        impl sdm_engine::sdm::EntitySet for $name {
            fn mode(&self) -> sdm_engine::sdm::EntitySetMode {
                self.mode
            }

            fn sort_container(&self) {
                self.container
                .borrow_mut()
                .sort_by(|a, b| a.1.priority().cmp(&b.1.priority()).reverse())
            }

            fn push(&self, entity: Box<dyn sdm_engine::sdm::Entity>) {
                let time = sdm_engine::sdm::Scheduler::time();
                match self.mode {
                    EntitySetMode::FIFO => self.container.borrow_mut().push((time, entity)),
                    EntitySetMode::LIFO => self.container.borrow_mut().insert(0, (time, entity)),
                    EntitySetMode::PRIORITY => {
                        self.container.borrow_mut().push((time, entity));
                        self.sort_container();
                    },
                }
            }

            fn pop(&self) -> Option<Box<dyn sdm_engine::sdm::Entity>> {
                if let Some((time, value)) = self.container.borrow_mut().pop() {
                    Some(value)
                } else {
                    None
                }
            }

            fn remove(&self, id: uuid::Uuid) -> Option<Box<dyn sdm_engine::sdm::Entity>> {
                let mut idx = None;
                for (i, elem) in self.container.borrow().iter().enumerate() {
                    if elem.1.id() == &id {
                        idx = Some(i);
                        break;
                    }
                }

                if let Some(i) = idx {
                    let (_, removed) = self.container.borrow_mut().remove(i);
                    return Some(removed);
                }

                None
            }

            fn is_empty(&self) -> bool {
                self.container.borrow().is_empty()
            }

            fn is_full(&self) -> bool {
                if let Some(size) = self.max_size {
                    self.container.borrow().len() == size as usize
                } else {
                    false
                }
            }

            fn apply_for_id<F: Fn(&dyn sdm_engine::sdm::Entity) -> ()>(&self, id: uuid::Uuid, func: F) -> anyhow::Result<()> {
                let mut idx = None;
                for (i, elem) in self.container.borrow().iter().enumerate() {
                    if elem.1.id() == &id {
                        idx = Some(i);
                        break;
                    }
                }

                if let Some(i) = idx {
                    func(self.container.borrow_mut()[i].1.as_ref());
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("No Entity matches provided ID"))
                }
            }

            fn size(&self) -> usize {
                self.container.borrow().len()
            }

            fn average_size(&self) -> f32 {
                let avg_size = self.average_size_sum.borrow().clone();
                avg_size as f32 / (sdm_engine::sdm::Scheduler::time() / sdm_engine::sdm::scheduler::ANALYTICS_REFRESH)
            }

            fn max_size(&self) -> Option<usize> {
                self.max_size
            }

            fn update_analytics(&self) {
                // Average size
                *self.average_size_sum.borrow_mut() += self.size() as u32;

                // Max time in set
                for (time_added, _) in self.container.borrow().iter() {
                    if *self.max_time_in_set.borrow() > sdm_engine::sdm::Scheduler::time() - time_added {
                        *self.max_time_in_set.borrow_mut() = sdm_engine::sdm::Scheduler::time() - time_added;
                    }
                }
            }

            fn average_time_in_set(&self) -> f32 {
                let mut average = 0f32;

                for (time_added, _) in self.container.borrow().iter() {
                    average += (sdm_engine::sdm::Scheduler::time() - time_added);
                }

                average
            }

            fn max_time_in_set(&self) -> f32 {
                self.max_time_in_set.borrow().clone()
            }
        }

        impl $name {
            pub fn new(name: &str, mode: sdm_engine::sdm::EntitySetMode $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    mode,
                    max_size: None,
                    average_size_sum: std::cell::RefCell::new(0u32),
                    max_time_in_set: std::cell::RefCell::new(0f32),
                    container: std::cell::RefCell::new(vec![]),
                    $($($varname,)*)?
                }
            }

            pub fn new_sized(name: &str, mode: sdm_engine::sdm::EntitySetMode, max_size: usize $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    mode,
                    max_size: Some(max_size),
                    average_size_sum: std::cell::RefCell::new(0u32),
                    max_time_in_set: std::cell::RefCell::new(0f32),
                    container: std::cell::RefCell::new(vec![]),
                    $($($varname,)*)?
                }
            }
        }
    };
}
