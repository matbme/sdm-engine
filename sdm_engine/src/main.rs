use sdm_engine::sdm::*;
use sdm_engine::*;

// use petri_engine::{net, petri_net};

use std::cell::RefCell;
use std::rc::Rc;

EntityWrapper! {
    pub struct Client {
        pub n_people: u32,
        pub served: bool,
    };
}

EntityWrapper! {
    pub struct Food {
        client_id: uuid::Uuid,
        done: bool,
    };
}

EntitySetWrapper! {
    pub struct FoodPreparationQueue;
}

EntitySetWrapper! {
    pub struct PreparedFoodQueue;
}

EventWrapper! {
    pub struct Arrival {
        n_people: u32,
        order_queue: Rc<dyn EntitySet>,
    };

    @execute = |event| {
        println!("{:.2} - Client arrived with size {}", Scheduler::time(), event.n_people);
        event.order_queue.push(Box::new(
            Client::new("Client", Scheduler::time(), event.n_people, false)
        ));

        Scheduler::instance().unwrap().schedule_in(
            Box::new(Arrival::new(
                "Arrival",
                f32::round(sdm::distributions::Uniform::gen(1.0, 4.0)) as u32,
                event.order_queue.clone()
            )),
            sdm::distributions::Uniform::gen(3.0, 20.0)
        );
    };
}

EntitySetWrapper! {
    pub struct OrderQueue;
}

ResourceWrapper! {
    pub struct Attendant;
}

EntitySetWrapper! {
    pub struct ClientsOrdering;
}

ProcessWrapper! {
    pub struct Order {
        ordering_clients: Rc<dyn EntitySet>,
        order_queue: Rc<dyn EntitySet>,
        food_prep_queue: Rc<dyn EntitySet>,
        attendants: Rc<dyn Resource>,
        counter_queue: Rc<dyn EntitySet>,
        tables_for_2_queue: Rc<dyn EntitySet>,
        tables_for_4_queue: Rc<dyn EntitySet>,
        did_allocate: RefCell<Vec<bool>>,
    };

    @on_start = |proc| {
        if !proc.order_queue.is_empty() {
            match proc.attendants.allocate(1) {
                Ok(()) => {
                    println!("{:.2} - Client started ordering", Scheduler::time());
                    proc.ordering_clients.push(proc.order_queue.pop().unwrap());
                    proc.did_allocate.borrow_mut().push(true);
                },
                Err(_) => {
                    proc.did_allocate.borrow_mut().push(false);
                }
            }
        } else {
            proc.did_allocate.borrow_mut().push(false);
        }
    };

    @on_end = |proc| {
        if proc.did_allocate.borrow_mut().remove(0) {
            println!("{:.2} - Client done ordering", Scheduler::time());
            proc.attendants.release(1);
            let client = proc.ordering_clients.pop().unwrap().downcast::<Client>().unwrap();

            proc.food_prep_queue.push(Box::new(
                Food::new("Food", Scheduler::time(), client.id().clone(), false)
            ));

            match client.n_people {
                1 => proc.counter_queue.push(client as Box<dyn Entity>),
                2 => proc.tables_for_2_queue.push(client as Box<dyn Entity>),
                3 ..= 4 => proc.tables_for_4_queue.push(client as Box<dyn Entity>),
                _ => unimplemented!()
            }
        }
    };
}

ResourceWrapper! {
    pub struct Cook;
}

EntitySetWrapper! {
    pub struct FoodInPreparationQueue;
}

ProcessWrapper! {
    pub struct PrepareFood {
        food_in_preparation: Rc<dyn EntitySet>,
        food_prep_queue: Rc<dyn EntitySet>,
        prepared_food_queue: Rc<dyn EntitySet>,
        cooks: Rc<dyn Resource>,
        did_allocate: RefCell<Vec<bool>>,
        // waiters: Rc<dyn EntitySet>,
    };

    @on_start = |proc| {
        if !proc.food_prep_queue.is_empty() {
            match proc.cooks.allocate(1) {
                Ok(()) => {
                    println!("{:.2} - Food being prepared", Scheduler::time());
                    proc.food_in_preparation.push(proc.food_prep_queue.pop().unwrap());
                    proc.did_allocate.borrow_mut().push(true);
                },
                Err(_) => {
                    proc.did_allocate.borrow_mut().push(false);
                }
            }
        } else {
            proc.did_allocate.borrow_mut().push(false);
        }
    };

    @on_end = |proc| {
        if proc.did_allocate.borrow_mut().remove(0) {
            println!("{:.2} - Food prepared", Scheduler::time());
            proc.cooks.release(1);
            let mut food = proc.food_in_preparation.pop().unwrap().downcast::<Food>().unwrap();
            food.done = true;
            proc.prepared_food_queue.push(food as Box<dyn Entity>);
        }

        // if let Some(waiter) = proc.waiters.pop() {
        //     waiter
        //         .downcast_ref::<Waiter>()
        //         .unwrap()
        //         .petri_net()
        //         .as_ref()
        //         .unwrap()
        //         .place_with_name("ORDER_READY")
        //         .unwrap()
        //         .add_tokens(1);
        //
        //     proc.waiters.push(waiter);
        // }
    };
}

ProcessWrapper! {
    pub struct ServeFood {
        prepared_food_queue: Rc<dyn EntitySet>,
        counter_seats: Rc<dyn Resource>,
        tables_for_2: Rc<dyn Resource>,
        tables_for_4: Rc<dyn Resource>,
    };

    @on_start = |proc| {
        if !proc.prepared_food_queue.is_empty() {
            println!("{:.2} - Food being served", Scheduler::time());
            let food = proc.prepared_food_queue.pop().unwrap().downcast::<Food>().unwrap();

            // Look for client
            for (i, client) in proc.counter_seats.downcast_ref::<TableSeats>().unwrap().clients.borrow_mut().iter_mut().enumerate() {
                if client.id() == &food.client_id {
                    client.downcast_mut::<Client>().unwrap().served = true;
                    Scheduler::instance().unwrap().schedule_in(
                        Box::new(Leave::new("Client leave", i, proc.counter_seats.clone())),
                        10.0
                    );
                    return
                }
            }

            for (i, client) in proc.tables_for_2.downcast_ref::<TableSeats>().unwrap().clients.borrow_mut().iter_mut().enumerate() {
                if client.id() == &food.client_id {
                    client.downcast_mut::<Client>().unwrap().served = true;
                    Scheduler::instance().unwrap().schedule_in(
                        Box::new(Leave::new("Client leave", i, proc.tables_for_2.clone())),
                        10.0
                    );
                    return
                }
            }

            for (i, client) in proc.tables_for_4.downcast_ref::<TableSeats>().unwrap().clients.borrow_mut().iter_mut().enumerate() {
                if client.id() == &food.client_id {
                    client.downcast_mut::<Client>().unwrap().served = true;
                    Scheduler::instance().unwrap().schedule_in(
                        Box::new(Leave::new("Client leave", i, proc.tables_for_4.clone())),
                        10.0
                    );
                    return
                }
            }
        }
    };
}

ProcessWrapper! {
    pub struct SeatClient {
        counter_queue: Rc<dyn EntitySet>,
        tables_for_2_queue: Rc<dyn EntitySet>,
        tables_for_4_queue: Rc<dyn EntitySet>,
        counter_seats: Rc<dyn Resource>,
        tables_for_2: Rc<dyn Resource>,
        tables_for_4: Rc<dyn Resource>,
    };

    @on_start = |proc| {
        if !proc.counter_queue.is_empty() {
            match proc.counter_seats.allocate(1) {
                Ok(()) => {
                    println!("{:.2} - Client being seated at counter", Scheduler::time());
                    proc.counter_seats
                        .downcast_ref::<TableSeats>()
                        .unwrap()
                        .clients
                        .borrow_mut()
                        .push(proc.counter_queue.pop().unwrap());
                }
                Err(_) => {}
            }
        }

        if !proc.tables_for_2_queue.is_empty() {
            match proc.tables_for_2.allocate(1) {
                Ok(()) => {
                    println!("{:.2} - Client being seated in a table for 2", Scheduler::time());
                    proc.tables_for_2
                        .downcast_ref::<TableSeats>()
                        .unwrap()
                        .clients
                        .borrow_mut()
                        .push(proc.tables_for_2_queue.pop().unwrap());
                }
                Err(_) => {}
            }
        }

        if !proc.tables_for_4_queue.is_empty() {
            match proc.tables_for_4.allocate(1) {
                Ok(()) => {
                    println!("{:.2} - Client being seated in a table for 4", Scheduler::time());
                    proc.tables_for_4
                        .downcast_ref::<TableSeats>()
                        .unwrap()
                        .clients
                        .borrow_mut()
                        .push(proc.tables_for_4_queue.pop().unwrap());
                }
                Err(_) => {}
            }
        }
    };
}

ResourceWrapper! {
    pub struct TableSeats {
        pub clients: RefCell<Vec<Box<dyn Entity>>>,
    };
}

EntitySetWrapper! {
    pub struct TableQueue;
}

EventWrapper! {
    pub struct Leave {
        client_pos: usize,
        seat: Rc<dyn Resource>,
    };

    @execute = |event| {
        println!("{:.2} - Client leaving", Scheduler::time());
        event.seat.downcast_ref::<TableSeats>().unwrap().clients.borrow_mut().remove(event.client_pos);
        event.seat.release(1);
    };
}

EntityWrapper! {
    pub struct Waiter;
}

EntitySetWrapper! {
    pub struct Waiters;
}

fn main() {
    if let Ok(scheduler) = Scheduler::new() {
        // Create waiters
        // let n_waiters = 3;
        // let waiters = scheduler.manage_entity_set(Waiters::new("Waiters", EntitySetMode::FIFO));
        // for i in 0..n_waiters {
        //     let mut new_waiter = Box::new(Waiter::new(&format!("Waiter {}", i), 0.0));
        //     let pn = petri_net! {
        //         places => [IDLE<1>, ORDER_READY, CLIENT_WILL_SEAT, ORDER_SERVED, TABLE_CLEANED, EXT_ORDER, EXT_CLEAN],
        //         transitions => [T_ORDER_START, T_ORDER_END, T_CLEAN_START, T_CLEAN_END],
        //         connections => [
        //             IDLE -> T_ORDER_START,
        //             IDLE -> T_CLEAN_START,
        //             ORDER_READY -> T_ORDER_START,
        //             CLIENT_WILL_SEAT -> T_CLEAN_START,
        //             T_ORDER_START -> ORDER_SERVED,
        //             T_CLEAN_START -> TABLE_CLEANED,
        //             ORDER_SERVED -> T_ORDER_END,
        //             TABLE_CLEANED -> T_CLEAN_END,
        //             EXT_ORDER -> T_ORDER_END,
        //             EXT_CLEAN -> T_CLEAN_END,
        //             T_ORDER_END -> IDLE,
        //             T_CLEAN_END -> IDLE
        //         ]
        //     };
        //
        //     new_waiter.add_petri_net(pn);
        //     waiters.push(new_waiter);
        // }

        // People
        let attendants = scheduler.manage_resource(Attendant::new("Attendants", 2));
        let cooks = scheduler.manage_resource(Cook::new("Cook", 3));

        // Order queues
        let order_queue =
            scheduler.manage_entity_set(OrderQueue::new("Order queue", EntitySetMode::FIFO));
        let ordering_clients =
            scheduler.manage_entity_set(OrderQueue::new("Ordering queue", EntitySetMode::FIFO));

        // Kitchen queues
        let food_prep_queue = scheduler
            .manage_entity_set(FoodPreparationQueue::new("Food waiting for prep", EntitySetMode::FIFO));
        let food_in_preparation_queue = scheduler
            .manage_entity_set(FoodPreparationQueue::new("Food in preparation", EntitySetMode::FIFO));
        let prepared_food_queue = scheduler
            .manage_entity_set(PreparedFoodQueue::new("Prepared Food", EntitySetMode::FIFO));

        // Table queues
        let counter_queue =
            scheduler.manage_entity_set(TableQueue::new("Counter queue", EntitySetMode::FIFO));
        let table_for_2_queue =
            scheduler.manage_entity_set(TableQueue::new("Table for 2 queue", EntitySetMode::FIFO));
        let table_for_4_queue =
            scheduler.manage_entity_set(TableQueue::new("Table for 4 queue", EntitySetMode::FIFO));

        // Tables
        let counter_seats =
            scheduler.manage_resource(TableSeats::new("Counter", 10, RefCell::new(vec![])));
        let tables_for_2 =
            scheduler.manage_resource(TableSeats::new("Tables for 2", 15, RefCell::new(vec![])));
        let tables_for_4 =
            scheduler.manage_resource(TableSeats::new("Tables for 4", 7, RefCell::new(vec![])));

        // First client (schedules more clients)
        let first_arrival = Arrival::new(
            "Client arrival",
            f32::round(sdm::distributions::Uniform::gen(1.0, 4.0)) as u32,
            order_queue.clone(),
        );
        scheduler.schedule_now(Box::new(first_arrival));

        // Processes
        scheduler.start_process_now(Box::new(Order::new(
            "Order",
            sdm::distributions::Uniform::new(1.0, 2.0),
            ordering_clients.clone(),
            order_queue.clone(),
            food_prep_queue.clone(),
            attendants.clone(),
            counter_queue.clone(),
            table_for_2_queue.clone(),
            table_for_4_queue.clone(),
            RefCell::new(vec![])
        )));
        scheduler.start_process_now(Box::new(PrepareFood::new(
            "Prepare Food",
            sdm::distributions::Uniform::new(10.0, 20.0), // CANNOT BE NEGATIVE
            food_in_preparation_queue.clone(),
            food_prep_queue.clone(),
            prepared_food_queue.clone(),
            cooks.clone(),
            RefCell::new(vec![])
        )));
        scheduler.start_process_now(Box::new(ServeFood::new(
            "Serve food",
            sdm::distributions::Uniform::new(1.0, 2.0),
            prepared_food_queue.clone(),
            counter_seats.clone(),
            tables_for_2.clone(),
            tables_for_4.clone()
        )));
        scheduler.start_process_now(Box::new(SeatClient::new(
            "Seat clients",
            sdm::distributions::Uniform::new(1.0, 2.0),
            counter_queue.clone(),
            table_for_2_queue.clone(),
            table_for_4_queue.clone(),
            counter_seats.clone(),
            tables_for_2.clone(),
            tables_for_4.clone()
        )));

        // Let's get this show on the road
        scheduler.simulate();
    }
}
