//! The phenomenon we wish to simulate

use rand::prelude::ThreadRng;
use rand::Rng;

/// Configuration for the simulation.
#[derive(Clone)]
pub struct Simulation {
    pub money_start: f64,
    pub chance_of_winning: f64,
    pub max_number_of_bets: i32,
    pub money_max: f64,
    pub bet_proportion: f64,
}

impl Simulation {
    /// Runs the simulation.
    /// 
    /// Plays a betting game like described in the Wikipedia article on Kelly
    /// criterion
    ///
    /// TODO: rename to run.
    pub fn run(self, rng: &mut ThreadRng) -> SimulationResult {
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

        SimulationResult {
            money_max,
            money,
            rounds,
            lost_all_money,
            has_max_prize,
        }
    }
}

/// The result from completing a task.
pub struct SimulationResult {
    pub money: f64,
    pub rounds: i32,
    pub money_max: f64,
    pub lost_all_money: bool,
    pub has_max_prize: bool,
}
