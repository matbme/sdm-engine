use std::cell::RefCell;

use anyhow::Result;

#[derive(Default)]
pub struct ResourceInner(pub RefCell<i32>);

pub trait Resource {
    fn allocate(&self, quantity: i32) -> Result<()>;

    fn release(&self, quantity: i32);

    fn update_analytics(&self);

    fn allocation_rate(&self) -> f32;

    fn average_allocation(&self) -> f32;
}


#[macro_export]
macro_rules! ResourceWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ; ) => {
        #[derive(Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            allocation_time: std::cell::RefCell<f32>,
            quantity: i32,
            times_allocated: std::cell::RefCell<u32>,
            tokens: sdm_engine::sdm::resource::ResourceInner,
            $($(
                $varname: $type,
            )*)?
        }

        impl sdm_engine::sdm::Resource for $name {
            fn allocate(&self, quantity: i32) -> anyhow::Result<()> {
                if quantity <= *self.tokens.0.borrow() {
                    *self.tokens.0.borrow_mut() -= quantity;
                    *self.times_allocated.borrow_mut() += 1;
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Not enough resources to allocate."))
                }
            }

            fn release(&self, quantity: i32) {
                *self.tokens.0.borrow_mut() += quantity;
            }

            fn update_analytics(&self) {
                // Allocation time
                if *self.tokens.0.borrow() < self.quantity {
                    *self.allocation_time.borrow_mut() += sdm_engine::sdm::scheduler::ANALYTICS_REFRESH;
                }
            }

            fn allocation_rate(&self) -> f32 {
                *self.allocation_time.borrow() / sdm_engine::sdm::Scheduler::time()
            }

            fn average_allocation(&self) -> f32 {
                *self.times_allocated.borrow() as f32 / sdm_engine::sdm::Scheduler::time()
            }
        }

        impl $name {
            pub fn new(name: &str, quantity: i32) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    allocation_time: std::cell::RefCell::new(0f32),
                    quantity: quantity,
                    times_allocated: std::cell::RefCell::new(0u32),
                    tokens: sdm_engine::sdm::resource::ResourceInner(std::cell::RefCell::new(quantity)),
                    $($($varname,)*)?
                }
            }
        }
    };
}
