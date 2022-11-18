use sdm_engine::*;
use sdm_engine::sdm::*;

// TODO: Remove "Wrapper" from macro names
EntityWrapper! {
    pub struct Carro {
        cor: String,
        marca: String,
        teste: i32,
    };
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
        fila: Fila,
    };

    @execute = |event| {
        println!("Chegada: {}", Scheduler::time());
    };
}

fn main() {

}
