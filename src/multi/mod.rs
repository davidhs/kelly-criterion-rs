//! Multi-threaded solution
//!
//! You break up a problem into multiple different different tasks and gather
//! the results into a solution.
//!
//! One thread is responsible for orchestrating the work and gathering the
//! results and then eventually returning the answer.
//!
//! Running `run` will block the orchestrating thread (main thread).
//!
//! The unit of work I call task.

mod orchestrator;
mod simulation;
mod task;
mod worker;

use simulation::Simulation;
use std::{collections::VecDeque};
use task::{Task, TaskResult};
use orchestrator::Orchestrator;

pub fn run_multi(repetitions: i32) {
    // let now = Instant::now();

    // Tasks are stored in a queue.
    let mut tasks: VecDeque<Task> = VecDeque::new();

    // Create tasks
    for i in 0..=100 {
        let id = i;
        let bet_proportion = (i as f64) / 100.0;

        let simulation_template = Simulation {
            money_start: 25.0,
            chance_of_winning: 0.60,
            max_number_of_bets: 300,
            money_max: 250.0,
            bet_proportion: bet_proportion,
        };

        let task = Task {
            id,
            repetitions: repetitions,
            simulation_template,
        };
        
        tasks.push_back(task);
    }

    // The main thread is the orchestrator and assigns tasks to workers.  Once
    // a worker completes its task it will send the result back to the
    // orchestrator.

    
    let mut orchestrator = Orchestrator::new(tasks);
    
    orchestrator.run();
    
    let task_results = orchestrator.get_results();
    
    // Duration: 8590 ms
    // println!("Duration: {} ms", now.elapsed().as_millis());
    
    // println!("Results: {:#?}", results);
    
    // TODO: worker pool needs to receive `to_worker_receiver` and `to_orchestrator_sender`

    // Display results
    for task_result in task_results {

        let bet_proportion = task_result.bet_proportion;
        let avg_money = task_result.avg_money;
        let prop_lost = task_result.prop_lost;
        let prop_maxed = task_result.prop_maxed;

        let bet_percent = (100.0 * bet_proportion).floor() as i64;
        let avg_money = avg_money;
        let prop_lost = 100.0 * prop_lost;
        let prop_maxed = 100.0 * prop_maxed;

        println!(
            "{}\t{}\t{}\t{}",
            bet_percent, avg_money, prop_lost, prop_maxed,
        );
    }
}

pub fn run_multi_single(repetitions: i32) {
    // let now = Instant::now();
    
    // let pool = ThreadPool::new(nr_of_cpus);

    // Create tasks
    let mut tasks: Vec<Task> = Vec::new();
    let mut task_results: Vec<Option<TaskResult>> = Vec::new();

    // 0..=100
    for i in 0..=100 {
        let id = i;
        let bet_proportion = (i as f64) / 100.0;

        let simulation_template = Simulation {
            money_start: 25.0,
            chance_of_winning: 0.60,
            max_number_of_bets: 300,
            money_max: 250.0,
            bet_proportion: bet_proportion,
        };

        tasks.push(Task {
            id,
            repetitions,
            simulation_template,
        });

        // Placeholder for task result.
        task_results.push(None);
    }

    // Do work
    for task in tasks {
        let task_result = task.run();

        let index = task_result.id;

        task_results[index] = Some(task_result);
    }
    
    // Duration: 102743 ms
    // println!("Duration: {} ms", now.elapsed().as_millis());

    // Display results
    for task_result in task_results {
        let task_result = task_result.unwrap();

        let bet_proportion = task_result.bet_proportion;
        let avg_money = task_result.avg_money;
        let prop_lost = task_result.prop_lost;
        let prop_maxed = task_result.prop_maxed;

        let bet_percent = (100.0 * bet_proportion).floor() as i64;
        let avg_money = avg_money;
        let prop_lost = 100.0 * prop_lost;
        let prop_maxed = 100.0 * prop_maxed;

        println!(
            "{}\t{}\t{}\t{}",
            bet_percent, avg_money, prop_lost, prop_maxed,
        );
    }
}
