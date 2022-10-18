use std::cell::RefCell;

use anyhow::{anyhow, Result};
use uuid::Uuid;

struct ResourceInner(RefCell<i32>);

pub struct Resource {
    name: String,
    id: Uuid,
    quantity: ResourceInner,
}

impl Resource {
    pub fn new(name: &str, quantity: i32) -> Self {
        Self {
            name: name.to_string(),
            id: Uuid::new_v4(),
            quantity: ResourceInner(RefCell::new(quantity)),
        }
    }

    pub fn allocate(&self, quantity: i32) -> Result<()> {
        if quantity <= *self.quantity.0.borrow() {
            *self.quantity.0.borrow_mut() -= quantity;
            Ok(())
        } else {
            Err(anyhow!("Not enough resources to allocate."))
        }
    }

    pub fn release(&self, quantity: i32) {
        *self.quantity.0.borrow_mut() += quantity;
    }
}

// TODO: Implement log
