use std::cell::RefCell;

use anyhow::{anyhow, Result};
use uuid::Uuid;

use super::entity::Entity;

#[derive(Clone, Copy)]
pub enum EntitySetMode {
    FIFO,
    LIFO,
    PRIORITY,
    NONE,
}

struct EntitySetContainer(RefCell<Vec<Entity>>);

pub struct EntitySet {
    name: String,
    id: Uuid,
    mode: EntitySetMode,
    max_size: Option<u32>,
    container: EntitySetContainer,
}

impl EntitySet {
    pub fn new(name: &str, mode: EntitySetMode) -> Self {
        Self {
            name: name.to_string(),
            id: Uuid::new_v4(),
            mode,
            max_size: None,
            container: EntitySetContainer(RefCell::new(vec![])),
        }
    }

    pub fn new_sized(name: &str, mode: EntitySetMode, max_size: u32) -> Self {
        Self {
            name: name.to_string(),
            id: Uuid::new_v4(),
            mode,
            max_size: Some(max_size),
            container: EntitySetContainer(RefCell::new(vec![])),
        }
    }

    pub fn mode(&self) -> EntitySetMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: EntitySetMode) {
        self.mode = mode;
    }

    pub fn push(&self, entity: Entity) {
        self.container.0.borrow_mut().push(entity);
    }

    pub fn pop(&self) -> Option<Entity> {
        self.container.0.borrow_mut().pop()
    }

    pub fn remove(&self, id: Uuid) -> Option<Entity> {
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

    pub fn is_empty(&self) -> bool {
        self.container.0.borrow().is_empty()
    }

    pub fn is_full(&self) -> bool {
        if let Some(size) = self.max_size {
            self.container.0.borrow().len() == size as usize
        } else {
            false
        }
    }

    pub fn apply_for_id<F: Fn(&Entity) -> ()>(&self, id: Uuid, func: F) -> Result<()> {
        let mut idx = None;
        for (i, elem) in self.container.0.borrow().iter().enumerate() {
            if elem.id() == &id {
                idx = Some(i);
                break;
            }
        }

        if let Some(i) = idx {
            func(&self.container.0.borrow_mut()[i]);
            Ok(())
        } else {
            Err(anyhow!("No Entity matches provided ID"))
        }
    }
}

// TODO: Implement log
