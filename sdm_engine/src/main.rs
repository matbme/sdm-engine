use sdm_engine::sdm::*;
use sdm_engine::*;

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
        fila: Rc<dyn EntitySet>,
        frentista: Rc<dyn Resource>,
    };

    @on_start = |proc| {
        if !proc.fila.is_empty() {
            if proc.frentista.allocate(1).is_ok() {
                println!("{} - Inicio abastecimento", Scheduler::time());
                proc.carro = proc.fila.pop().unwrap();
            }
        }
    };

    @on_end = |proc| {
        println!("{} - Fim abastecimento", Scheduler::time());
        proc.frentista.release(1);
    };
}

EventWrapper! {
    pub struct Chegada {
        time_limit: f32,
        fila: Rc<dyn EntitySet>,
    };

    @execute = |event| {
        if Scheduler::time() < event.time_limit {
            println!("{} - Chegada", Scheduler::time());
            // Schedules new arrival in 5s
            Scheduler::instance()
                .unwrap()
                .schedule_in(
                    Box::new(Chegada::new("Chegada", 100.0, event.fila.clone())),
                    5.0
                );

            // Adds car to queue
            event.fila.push(Box::new(Carro::new("Carro", Scheduler::time())))
        }
    };
}

fn main() {
    if let Ok(scheduler) = Scheduler::new() {
        let frentista = scheduler.manage_resource(Frentista::new("Frentista", 2));
        let fila_posto =
            scheduler.manage_entity_set(Fila::new_sized("Abastecimento", EntitySetMode::FIFO, 100));

        let chegada = Chegada::new("Chegada", 100.0, fila_posto.clone());
        let abastecimento = Abastecimento::new(
            "Abastecimento",
            distributions::Gaussian::new(8.0, 2.0),
            Box::new(Carro::new("Vazio", 0.0)),
            fila_posto.clone(),
            frentista.clone(),
        );

        scheduler.schedule_now(Box::new(chegada));
        scheduler.start_process_now(Box::new(abastecimento));

        scheduler.simulate();
    }
}
