mod orchestra;
mod simulation;

use simulation::Simulation;
use std::env;
use std::time::Instant;

pub fn run(repetitions: i32) {
    let now = Instant::now();

    // Create simulations to run.
    let mut simulations: Vec<Simulation> = Vec::new();

    for i in 0..=100 {
        let bet_proportion = (i as f64) / 100.0;

        let simulation = Simulation {
            money_start: 25.0,
            chance_of_winning: 0.60,
            max_number_of_bets: 300,
            money_max: 250.0,
            bet_proportion: bet_proportion,
            repetitions,
        };

        simulations.push(simulation);
    }

    let simulation_results = orchestra::run(simulations);

    println!(
        "{}\t{}\t{}\t{}\n",
        "Bet (%)", "Avg. money", "Prop. lost", "Prop. maxed",
    );

    // Display results
    for simulation_result in simulation_results {
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

    println!("\nTime elapsed: {} s", now.elapsed().as_secs_f64());
}

fn main() {
    let mut args = env::args().skip(1);

    if let Some(arg) = args.next() {
        let repetitions: i32 = arg.parse().unwrap();

        assert!(repetitions >= 1);

        run(repetitions);

        return;
    }

    println!("app REPETITIONS");
}
