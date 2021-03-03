mod orchestra;
mod simulation;

use orchestra::Task;
use simulation::Simulation;
use std::env;
use std::time::Instant;

pub fn run(repetitions: i32) {
    // Tasks are stored in a queue.
    let mut tasks: Vec<Task> = Vec::new();

    // Create tasks
    for i in 0..=100 {
        let id = i;
        let bet_proportion = (i as f64) / 100.0;

        let simulation = Simulation {
            money_start: 25.0,
            chance_of_winning: 0.60,
            max_number_of_bets: 300,
            money_max: 250.0,
            bet_proportion: bet_proportion,
            repetitions,
        };

        let task = Task { id, simulation };

        tasks.push(task);
    }

    let task_results = orchestra::run(tasks);

    println!(
        "{}\t{}\t{}\t{}\n",
        "Bet (%)", "Avg. money", "Prop. lost", "Prop. maxed",
    );

    // Display results
    for task_result in task_results {
        let simulation_result = task_result.simulation_result;

        let bet_proportion = simulation_result.bet_proportion;
        let avg_money = simulation_result.avg_money;
        let prop_lost = simulation_result.prop_lost;
        let prop_maxed = simulation_result.prop_maxed;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let repetitions: i32 = args[1].parse().expect("Repetitions should be an integer.");

        let now = Instant::now();

        run(repetitions);

        println!("\nTime elapsed: {} s", now.elapsed().as_secs_f64());

        return;
    }

    println!("Usage: cargo run REPETITIONS");
}
