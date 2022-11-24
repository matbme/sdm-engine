use downcast_rs::{Downcast, impl_downcast};
use uuid::Uuid;
use petri_engine::net::PetriNet;

pub trait Entity: Downcast + std::fmt::Debug {
    fn id(&self) -> &Uuid;

    fn priority(&self) -> &Option<i32>;

    fn time_since_creation(&self) -> f32;

    fn set_priority(&mut self, priority: i32);

    fn clear_priority(&mut self);

    fn add_petri_net(
        &mut self,
        petri_net: PetriNet,
    ) -> Option<PetriNet>;

    fn petri_net(&self) -> &Option<PetriNet>;
}

impl_downcast!(Entity);

#[macro_export]
macro_rules! EntityWrapper {
    ( $vis:vis struct $name:ident $({ $($varvis:vis $varname:ident : $type:ty),* $(,)? })? ; ) => {
        #[derive(Debug, Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            priority: Option<i32>,
            petri_net: Option<petri_engine::net::PetriNet>,
            creation_time: f32,
            $($(
                $varvis $varname: $type,
            )*)?
        }

        impl sdm_engine::sdm::Entity for $name {
            fn id(&self) -> &uuid::Uuid {
                &self.id
            }

            fn priority(&self) -> &Option<i32> {
                &self.priority
            }

            fn time_since_creation(&self) -> f32 {
                sdm_engine::sdm::Scheduler::time() - self.creation_time
            }

            fn set_priority(&mut self, priority: i32) {
                self.priority = Some(priority);
            }

            fn clear_priority(&mut self) {
                self.priority = None;
            }

            fn add_petri_net(&mut self, petri_net: petri_engine::net::PetriNet) -> Option<petri_engine::net::PetriNet> {
                self.petri_net.replace(petri_net)
            }

            fn petri_net(&self) -> &Option<petri_engine::net::PetriNet> {
                &self.petri_net
            }
        }

        impl $name {
            pub fn new(name: &str, creation_time: f32 $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    priority: None,
                    petri_net: None,
                    creation_time,
                    $($($varname,)*)?
                }
            }

            pub fn new_with_priority(name: &str, priority: i32, creation_time: f32 $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    priority: Some(priority),
                    petri_net: None,
                    creation_time,
                    $($($varname,)*)?
                }
            }
        }
    };
}
