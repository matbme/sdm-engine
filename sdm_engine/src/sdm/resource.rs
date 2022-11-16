use std::cell::RefCell;

use anyhow::Result;

#[derive(Default)]
pub struct ResourceInner(pub RefCell<i32>);

pub trait Resource {
    fn allocate(&self, quantity: i32) -> Result<()>;

    fn release(&self, quantity: i32);
}


#[macro_export]
macro_rules! ResourceWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ; ) => {
        #[derive(Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            quantity: sdm_engine::sdm::resource::ResourceInner,
            $($(
                $varname: $type,
            )*)?
        }

        impl sdm_engine::sdm::Resource for $name {
            fn allocate(&self, quantity: i32) -> anyhow::Result<()> {
                if quantity <= *self.quantity.0.borrow() {
                    *self.quantity.0.borrow_mut() -= quantity;
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Not enough resources to allocate."))
                }
            }

            fn release(&self, quantity: i32) {
                *self.quantity.0.borrow_mut() += quantity;
            }
        }

        impl $name {
            pub fn new(name: &str, quantity: i32) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    quantity: sdm_engine::sdm::resource::ResourceInner(std::cell::RefCell::new(quantity)),
                    $($($varname,)*)?
                }
            }
        }
    };
}

// TODO: Implement log
