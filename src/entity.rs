use petri_engine::net::PetriNet;
use uuid::Uuid;

pub struct Entity {
    name: String,
    id: Uuid,
    creation_time: f32,
    priority: Option<i32>,
    petri_net: Option<PetriNet>,
}

impl Entity {
    pub fn new(name: &str, creation_time: f32) -> Self {
        Self {
            name: name.to_string(),
            id: Uuid::new_v4(),
            creation_time,
            priority: None,
            petri_net: None,
        }
    }

    pub fn new_with_pn(petri_net: PetriNet, name: &str, creation_time: f32) -> Self {
        Self {
            name: name.to_string(),
            id: Uuid::new_v4(),
            creation_time,
            priority: None,
            petri_net: Some(petri_net),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn priority(&self) -> &Option<i32> {
        &self.priority
    }

    pub fn time_since_creation(&self) -> f32 {
        todo!()
    }

    pub fn add_petri_net(&mut self, petri_net: PetriNet) -> Option<PetriNet> {
        self.petri_net.replace(petri_net)
    }

    pub fn petri_net(&self) -> &Option<PetriNet> {
        &self.petri_net
    }
}
