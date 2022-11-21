use sdm_engine::*;
use sdm_engine::sdm::*;

use std::rc::Rc;

// TODO: Remove "Wrapper" from macro names
EntityWrapper! {
    pub struct Carro;
}

ResourceWrapper! {
    pub struct Frentista;
}

EntitySetWrapper! {
    pub struct Fila;
}

ProcessWrapper! {
    pub struct Abastecimento {
        carro: Box<dyn Entity>,
        fila: Fila,
        frentista: Frentista,
    };

    @on_start = |proc| {
        if proc.fila.is_empty() {
            if proc.frentista.allocate(1).is_ok() {
                println!("Inicio abastecimento: {}", Scheduler::time());
                proc.carro = proc.fila.pop().unwrap();
            }
        }
    };

    @on_end = |proc| {

    };
}

EventWrapper! {
    pub struct Chegada {
        time_limit: f32,
        fila: Rc<Fila>,
    };

    @execute = |event| {
        println!("Chegada: {}", Scheduler::time());
        if Scheduler::time() < event.time_limit {
            // Schedules new arrival in 5s
            Scheduler::instance()
                .unwrap()
                .schedule_in(
                    Box::new(Chegada::new("Chegada", 100.0, event.fila.clone())),
                    5.0
                );

            // Adds car to queue
            event.fila.push(Carro::new("Carro", Scheduler::time()))
        }
    };
}

fn main() {

}
