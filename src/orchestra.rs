//! A module that orchestrates the computation of multiple simulations to multiple workers.

use crate::simulation::{Simulation, SimulationResult};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::{channel, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// A message that is intended for the orchestrator of the workers.
enum ToOrchestratorMessage {
    TaskResult(TaskResult),
}

/// Runs the orchestra
pub fn run(simulations: Vec<Simulation>) -> Vec<SimulationResult> {
    // Package up each simulation into a task.
    let mut tasks: Vec<Task> = Vec::new();
    {
        let mut id = 0;
        for simulation in simulations {
            tasks.push(Task { id, simulation });
            id += 1;
        }
    }

    // Create workers, a single worker for each logical core of your machine.

    let nr_of_workers = num_cpus::get();
    let mut nr_of_available_workers = nr_of_workers;

    println!("Creating {} workers...", nr_of_workers);

    // Set up channels so workers and the orchestrator can communicate.

    // A channel to send a task from a single orchestrator to multiple workers.
    let (to_worker_sender, to_worker_receiver) = channel::<ToWorkerMessage>();

    // A channel to send a task result from multiple works to a single orchestrator.
    let (to_orchestrator_sender, to_orchestrator_receiver) = channel::<ToOrchestratorMessage>();

    let to_worker_receiver = Arc::new(Mutex::new(to_worker_receiver));
    let to_orchestrator_sender = Arc::new(Mutex::new(to_orchestrator_sender));

    // Create workers
    
    let mut workers = Vec::with_capacity(nr_of_workers);

    for id in 0..nr_of_workers {
        let worker = Worker::new(
            id,
            Arc::clone(&to_worker_receiver),
            Arc::clone(&to_orchestrator_sender),
        );
        workers.push(worker);
    }

    let mut task_results = Vec::new();

    // Start working on the tasks.  This loop runs until all the tasks are complete and all the workers are free.
    loop {
        // Check if all workers are available.
        let all_available = nr_of_workers == nr_of_available_workers;

        // Check if we're done.  If there are no more pending tasks and all workers
        // are done then we've finished all the tasks.
        if tasks.is_empty() && all_available {
            break;
        }

        // This loop tries to assign as many tasks as possibles to workers if we have both
        // remaining tasks and free workers.
        while !tasks.is_empty() {
            let none_available = nr_of_available_workers == 0;
            if none_available {
                // Stop this loop and wait on receiving a message from a worker
                // to free a worker.
                break;
            }

            // Dequeue task from queue.
            let task = tasks.pop().expect("");

            // Send task to worker.
            to_worker_sender
                .send(ToWorkerMessage::Task(task))
                .expect("");
            nr_of_available_workers -= 1;
        }

        // At this point we have to wait for a message from a worker before we can continue.
        // Once a message is received that worker is free.
        let mut msg = to_orchestrator_receiver.recv().expect("");
        nr_of_available_workers += 1;

        // Process message.
        match msg {
            ToOrchestratorMessage::TaskResult(task_result) => {
                task_results.push(task_result);
            }
        }

        // In this loop we check if there are more messages from the workers.  We don't wait
        // for messages, just check if there are some waiting for us to respond to.  We process
        // the messages.  Once we find no more messages waiting for us this loop stops.
        loop {
            match to_orchestrator_receiver.try_recv() {
                Ok(message) => {
                    msg = message;

                    // Process message.
                    match msg {
                        ToOrchestratorMessage::TaskResult(task_result) => {
                            task_results.push(task_result);
                        }
                    }

                    nr_of_available_workers += 1;
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {
                            // Stop looping.
                            break;
                        }
                        TryRecvError::Disconnected => {
                            // TODO: I don't remember what this was supposed to do.  When
                            // can this happen?
                            todo!();
                        }
                    }
                }
            }
        }
    }

    // Clean up.

    // Send messages to workers to tell them to terminate.

    for _ in &workers {
        to_worker_sender.send(ToWorkerMessage::Terminate).unwrap();
    }

    // Wait until all workers (threads) have terminated.

    let mut workers = workers;

    for worker in &mut workers {
        if let Some(thread) = worker.thread.take() {
            thread.join().unwrap();
        }
    }

    // Sort the task results according to the task ID in ascending order.

    task_results.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

    // Place results into simulation results and return it.

    let mut simulation_results: Vec<SimulationResult> = Vec::new();

    for task_result in task_results {
        let simulation_result = task_result.simulation_result;
        simulation_results.push(simulation_result);
    }

    simulation_results
}

/// A task is the unit of work a thread works on.
pub struct Task {
    /// The ID of this task, just some number to uniquely identify this task.
    pub id: usize,
    // Configuration for simulation
    pub simulation: Simulation,
}

impl Task {
    /// Runs the task and returns a task result.  Running task consumes the task.
    fn run(self) -> TaskResult {
        TaskResult {
            id: self.id,
            simulation_result: self.simulation.run(),
        }
    }
}

#[derive(Debug)]
pub struct TaskResult {
    /// The result for task with the corresponding ID.
    id: usize,
    /// Result of running simulation
    pub simulation_result: SimulationResult,
}

/// Messages that are intended for the worker.
enum ToWorkerMessage {
    /// A worker gets assigned a new task.
    Task(Task),
    /// A worker should terminate
    Terminate,
}

struct Worker {
    /// ID of worker.
    // id: usize,
    /// The optional handle to a thread.
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker that starts spinning right away and waits for message.
    /// The message either contains a task to work on or it tells the worker to shut down.
    fn new(
        worker_id: usize,
        to_worker_receiver: Arc<Mutex<Receiver<ToWorkerMessage>>>,
        to_orchestrator_sender: Arc<Mutex<Sender<ToOrchestratorMessage>>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Wait for message from orchestrator.
                let message: ToWorkerMessage = to_worker_receiver
                    .lock() // acquire the mutex
                    .expect("Worker: mutex to receive message to worker in a poisoned state.") // might fail if the mutex is in a poisoned state (e.g. other thread paniced while holding the lock)
                    .recv() // receive message from channel
                    .expect("Worker: sending side of channel to worker is down.") // might fail, e.g. thread on sending side of the channel might have shut down
                ;

                // Carry out the command in the message.
                match message {
                    ToWorkerMessage::Task(task) => {
                        let task_id = task.id;
                        println!("> Worker {} working on task {}...", worker_id, task_id);

                        let task_result = task.run();
                        println!("< Worker {} completed task {}.", worker_id, task_id);

                        let wrapped_task_result = ToOrchestratorMessage::TaskResult(task_result);

                        // TODO: once the task is complete send the result to orchestrator

                        to_orchestrator_sender
                            .lock()
                            .expect("Worker: mutex to send message to orchestrator in a poisoned state.")
                            .send(wrapped_task_result)
                            .expect("Worker: receiving side of channel to orchestrator down.");
                    }
                    ToWorkerMessage::Terminate => {
                        println!("Worker {} terminating...", worker_id);
                        break;
                    }
                }
            }
        });

        Worker {
            // id: worker_id,
            thread: Some(thread),
        }
    }
}
