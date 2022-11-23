use std::cell::RefCell;

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

#[allow(dead_code)]
#[derive(Default)]
pub struct EntitySetContainer(pub RefCell<Vec<Box<dyn Entity>>>);

pub trait EntitySet {
    fn mode(&self) -> EntitySetMode;

    fn sort_container(&self);

    fn push(&self, entity: impl Entity + 'static);

    fn pop(&self) -> Option<Box<dyn Entity>>;

    fn remove(&self, id: Uuid) -> Option<Box<dyn Entity>>;

    fn is_empty(&self) -> bool;

    fn is_full(&self) -> bool;

    fn apply_for_id<F: Fn(&dyn Entity) -> ()>(&self, id: Uuid, func: F) -> Result<()>;
}

#[macro_export]
macro_rules! EntitySetWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ; ) => {
        #[derive(Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            mode: sdm_engine::sdm::EntitySetMode,
            max_size: Option<u32>,
            container: sdm_engine::sdm::entity_set::EntitySetContainer,
            $($(
                $varname: $type,
            )*)?
        }

        impl sdm_engine::sdm::EntitySet for $name {
            fn mode(&self) -> sdm_engine::sdm::EntitySetMode {
                self.mode
            }

            fn sort_container(&self) {
                self.container.
                0
                .borrow_mut()
                .sort_by(|a, b| a.priority().cmp(&b.priority()).reverse())
            }

            fn push(&self, entity: impl sdm_engine::sdm::Entity + 'static) {
                match self.mode {
                    EntitySetMode::FIFO => self.container.0.borrow_mut().push(Box::new(entity)),
                    EntitySetMode::LIFO => self.container.0.borrow_mut().insert(0, Box::new(entity)),
                    EntitySetMode::PRIORITY => {
                        self.container.0.borrow_mut().push(Box::new(entity));
                        self.sort_container();
                    },
                }
            }

            fn pop(&self) -> Option<Box<dyn sdm_engine::sdm::Entity>> {
                self.container.0.borrow_mut().pop()
            }

            fn remove(&self, id: uuid::Uuid) -> Option<Box<dyn sdm_engine::sdm::Entity>> {
                let mut idx = None;
                for (i, elem) in self.container.0.borrow().iter().enumerate() {
                    if elem.id() == &id {
                        idx = Some(i);
                        break;
                    }
                }

                if let Some(i) = idx {
                    return Some(self.container.0.borrow_mut().remove(i));
                }

                None
            }

            fn is_empty(&self) -> bool {
                self.container.0.borrow().is_empty()
            }

            fn is_full(&self) -> bool {
                if let Some(size) = self.max_size {
                    self.container.0.borrow().len() == size as usize
                } else {
                    false
                }
            }

            fn apply_for_id<F: Fn(&dyn sdm_engine::sdm::Entity) -> ()>(&self, id: uuid::Uuid, func: F) -> anyhow::Result<()> {
                let mut idx = None;
                for (i, elem) in self.container.0.borrow().iter().enumerate() {
                    if elem.id() == &id {
                        idx = Some(i);
                        break;
                    }
                }

                if let Some(i) = idx {
                    func(self.container.0.borrow_mut()[i].as_ref());
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("No Entity matches provided ID"))
                }
            }
        }

        impl $name {
            pub fn new(name: &str, mode: sdm_engine::sdm::EntitySetMode $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    mode,
                    max_size: None,
                    container: sdm_engine::sdm::entity_set::EntitySetContainer(std::cell::RefCell::new(vec![])),
                    $($($varname,)*)?
                }
            }

            pub fn new_sized(name: &str, mode: sdm_engine::sdm::EntitySetMode, max_size: u32 $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    mode,
                    max_size: Some(max_size),
                    container: sdm_engine::sdm::entity_set::EntitySetContainer(std::cell::RefCell::new(vec![])),
                    $($($varname,)*)?
                }
            }
        }
    };
}

// TODO: Implement log
