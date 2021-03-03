//! Each worker gets assigned a task.  A task can run one or more simulations.
//! Once the worker has completed a task another task will be assigned if more
//! work needs to be done (i.e. there are more tasks remaining).

use crate::multi::simulation::Simulation;

/// In this multi-threaded solution, a task is the unit of work a thread works
/// on.
pub struct Task {
    /// The ID of this task, just some number to uniquely identify this task.
    pub id: usize,
    /// How often we repeat a simulation to average out the result.
    pub repetitions: i32,
    // Configuration for simulation
    pub simulation_template: Simulation,
}

impl Task {
    /// Running task consumes the task.
    pub fn run(self) -> TaskResult {
        let simulation_template = self.simulation_template;

        let mut avg_money = 0.0;
        let mut avg_rounds = 0;
        let mut prop_lost = 0.0;
        let mut prop_maxed = 0.0;

        // TODO: this can be supplied by the worker!
        let mut rng = rand::thread_rng();

        for _ in 0..self.repetitions {
            let simulation = simulation_template.clone();
            let simulation_result = simulation.run(&mut rng);

            if simulation_result.lost_all_money {
                prop_lost += 1.0;
            }

            if simulation_result.has_max_prize {
                prop_maxed += 1.0;
            }

            avg_money += simulation_result.money;
            avg_rounds += simulation_result.rounds;
        }

        let repetitions_i32: i32 = self.repetitions;
        let repetitions_f64: f64 = self.repetitions as f64;

        avg_money /= repetitions_f64;
        avg_rounds /= repetitions_i32;
        prop_lost /= repetitions_f64;
        prop_maxed /= repetitions_f64;

        let id = self.id;

        TaskResult {
            id,
            avg_money,
            avg_rounds,
            prop_lost,
            prop_maxed,
            bet_proportion: simulation_template.bet_proportion,
        }
    }
}

#[derive(Debug)]
pub struct TaskResult {
    /// The result for task with the corresponding ID.
    pub id: usize,
    /// How much money was made on average.
    pub avg_money: f64,
    /// How many round on average we played per simulation
    pub avg_rounds: i32,
    /// Proportion of simulations that lost all money.
    pub prop_lost: f64,
    /// Proportion of simulations that maxed out on money.
    pub prop_maxed: f64,
    /// Proportion of our money we bet each time.
    pub bet_proportion: f64,
}
