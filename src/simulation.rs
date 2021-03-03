use rand::Rng;

/// Configuration for the simulation.
#[derive(Clone)]
pub struct Simulation {
    pub money_start: f64,
    pub chance_of_winning: f64,
    pub max_number_of_bets: i32,
    pub money_max: f64,
    pub bet_proportion: f64,
    pub repetitions: i32,
}

/// The result from completing a task.
#[derive(Debug)]
pub struct SimulationResult {
    pub avg_money: f64,
    pub avg_rounds: i32,
    pub prop_lost: f64,
    pub prop_maxed: f64,
    pub bet_proportion: f64,
}

impl Simulation {
    /// Runs the simulation.
    ///
    /// Plays a betting game like described in the Wikipedia article on Kelly
    /// criterion
    ///
    /// TODO: rename to run.
    pub fn run(self) -> SimulationResult {
        let mut avg_money = 0.0;
        let mut avg_rounds = 0;
        let mut prop_lost = 0.0;
        let mut prop_maxed = 0.0;

        // TODO: this can be supplied by the worker!
        let mut rng = rand::thread_rng();

        for _ in 0..self.repetitions {
            // Run "simulation"

            let money_start = self.money_start;
            let chance_of_winning = self.chance_of_winning;
            let max_number_of_bets = self.max_number_of_bets;
            let money_max = self.money_max;

            // Play until you:
            //
            // * you go bust (have no more money),
            // * you don't get to place any more bets, or
            // * you get the maximum amount of money.
            //

            let mut money = money_start;
            let mut rounds = 0;

            let mut lost_all_money = money <= std::f64::EPSILON;
            let mut has_max_prize = money >= (money_max - std::f64::EPSILON);

            for _ in 0..max_number_of_bets {
                let bet = self.bet_proportion * money;
                rounds += 1;

                if rng.gen_range(0.0..1.0) < chance_of_winning {
                    // Won
                    money += bet;
                } else {
                    // Lost
                    money -= bet;
                }

                lost_all_money = money <= std::f64::EPSILON;
                has_max_prize = money >= (money_max - std::f64::EPSILON);

                if lost_all_money {
                    money = money_start;
                    break;
                }

                if has_max_prize {
                    money = money_max;
                    break;
                }
            }

            if lost_all_money {
                prop_lost += 1.0;
            }

            if has_max_prize {
                prop_maxed += 1.0;
            }

            avg_money += money;
            avg_rounds += rounds;
        }

        let repetitions_i32: i32 = self.repetitions;
        let repetitions_f64: f64 = self.repetitions as f64;

        avg_money /= repetitions_f64;
        avg_rounds /= repetitions_i32;
        prop_lost /= repetitions_f64;
        prop_maxed /= repetitions_f64;

        SimulationResult {
            avg_money,
            avg_rounds,
            prop_lost,
            prop_maxed,
            bet_proportion: self.bet_proportion,
        }
    }
}
