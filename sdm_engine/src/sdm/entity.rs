use uuid::Uuid;
use petri_engine::net::PetriNet;

pub trait Entity {
    fn id(&self) -> &Uuid;

    fn priority(&self) -> &Option<i32>;

    fn time_since_creation(&self) -> f32;

    fn add_petri_net(
        &mut self,
        petri_net: PetriNet,
    ) -> Option<PetriNet>;

    fn petri_net(&self) -> &Option<PetriNet>;
}

#[macro_export]
macro_rules! EntityWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ; ) => {
        #[derive(Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            priority: Option<i32>,
            petri_net: Option<petri_engine::net::PetriNet>,
            creation_time: f32,
            $($(
                $varname: $type,
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
                todo!()
            }

            fn add_petri_net(&mut self, petri_net: petri_engine::net::PetriNet) -> Option<petri_engine::net::PetriNet> {
                self.petri_net.replace(petri_net)
            }

            fn petri_net(&self) -> &Option<petri_engine::net::PetriNet> {
                &self.petri_net
            }
        }

        impl $name {
            pub fn new(name: String, creation_time: f32 $(,$($varname: $type),*)?) -> Self {
                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    priority: None,
                    petri_net: None,
                    creation_time,
                    $($($varname,)*)?
                }
            }
        }
    };
}
