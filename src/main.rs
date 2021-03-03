mod multi;


use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() == 3 {
        
        
        
        let repetitions: i32 = args[2].parse().expect("Repetitions should be an integer.");
        
        if args[1] == "single" {
            
            
            return;
        }
        else if args[1] == "multi-single" {
            multi::run_multi_single(repetitions);
            return;
        } 
        else if args[1] == "multi" {
            multi::run_multi(repetitions);
            return;
        }
    }
    
    println!("{:?}", args);
    // single::run();
    // multi::run();
    
    println!("Usage: cargo run

single REPETITIONS
multi-single REPETITIONS
multi REPETITIONS
");
}
