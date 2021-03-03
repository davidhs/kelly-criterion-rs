use crate::simulation::{Simulation, SimulationResult};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::{channel, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub enum ToOrchestratorMessage {
    TaskResult(TaskResult),
}

/// NOTE: only ever interact with the workers through the worker manager, never
/// the workers directly.

/// Runs the orchestra
pub fn run(mut tasks: Vec<Task>) -> Vec<TaskResult> {
    let nr_of_workers = num_cpus::get();
    let mut nr_of_available_workers = nr_of_workers;

    // Send a task from a single orchestrator to multiple workers.
    let (to_worker_sender, to_worker_receiver) = channel::<ToWorkerMessage>();

    // Send a task result from multiple works to a single orchestrator
    let (to_orchestrator_sender, to_orchestrator_receiver) = channel::<ToOrchestratorMessage>();

    // All workers ...
    let to_worker_receiver = Arc::new(Mutex::new(to_worker_receiver));
    let to_orchestrator_sender = Arc::new(Mutex::new(to_orchestrator_sender));

    let mut workers = Vec::with_capacity(nr_of_workers);

    for _ in 0..nr_of_workers {
        let worker = Worker::new(
            Arc::clone(&to_worker_receiver),
            Arc::clone(&to_orchestrator_sender),
        );
        workers.push(worker);
    }

    let mut task_results = Vec::new();

    // Run task and task result loop.use task::Task;
    loop {
        let all_available = nr_of_workers == nr_of_available_workers;

        // Check if we're done.
        if tasks.is_empty() && all_available {
            break;
        }

        while !tasks.is_empty() {
            let none_available = nr_of_available_workers == 0;
            if none_available {
                // Stop this loop and wait on receiving a message from a worker
                // to free a worker.
                break;
            }

            // Dequeue task from queue.
            let task = tasks.pop().expect("");

            to_worker_sender
                .send(ToWorkerMessage::Task(task))
                .expect("");
            nr_of_available_workers -= 1;
        }

        // TODO: block on receiving message from worker
        let mut msg = to_orchestrator_receiver.recv().expect("");
        nr_of_available_workers += 1;

        match msg {
            ToOrchestratorMessage::TaskResult(task_result) => {
                task_results.push(task_result);
            }
        }

        loop {
            // NOTE: once a message is received from a worker then that worker is
            // freed and available.

            // TODO: update available workers queue

            // TODO: Handle message (message_to_handle)

            // TODO: check to see if there are more messages from the worker, if
            // so handle them, until there are no more messages to handle.

            match to_orchestrator_receiver.try_recv() {
                Ok(message) => {
                    // TODO: does this free the worker?
                    msg = message;

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
                            todo!();
                        }
                    }
                }
            }
        }
    }

    // TODO: drop what needs to be dropped!
    // println!("Sending terminate message to all workers.");

    for _ in &workers {
        to_worker_sender.send(ToWorkerMessage::Terminate).unwrap();
    }

    // println!("Shutting down all workers.");orchestrator

    let mut workers = workers;

    for worker in &mut workers {
        // println!("Shutting down worker {}", worker.id);

        if let Some(thread) = worker.thread.take() {
            thread.join().unwrap();
        }
    }

    task_results.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

    // TODO: use typestate pattern (?)

    // TODO: dispose of wall workers

    // TODO: return result or display result

    // todo!();

    task_results
}

// TODO: Simulation and Task are to intertwined.

/// In this multi-threaded solution, a task is the unit of work a thread works
/// on.
pub struct Task {
    /// The ID of this task, just some number to uniquely identify this task.
    pub id: usize,
    // Configuration for simulation
    pub simulation: Simulation,
}

impl Task {
    /// Running task consumes the task.
    pub fn run(self) -> TaskResult {
        TaskResult {
            id: self.id,
            simulation_result: self.simulation.run(),
        }
    }
}

#[derive(Debug)]
pub struct TaskResult {
    /// The result for task with the corresponding ID.
    pub id: usize,
    /// Result of running simulation
    pub simulation_result: SimulationResult,
}

/// Messages that are intended for the worker.
pub enum ToWorkerMessage {
    /// A worker gets assigned a new task.
    Task(Task),
    /// A worker should terminate
    Terminate,
}

pub struct Worker {
    /// ID of worker.
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker that starts spinning right away and awaits new
    /// tasks to execute.
    pub fn new(
        to_worker_receiver: Arc<Mutex<Receiver<ToWorkerMessage>>>,
        to_orchestrator_sender: Arc<Mutex<Sender<ToOrchestratorMessage>>>,
    ) -> Worker {
        // TODO: the random number generator can be created here.

        let thread = thread::spawn(move || {
            loop {
                let message: ToWorkerMessage = to_worker_receiver
                    .lock() // acquire the mutex
                    .expect("Worker: mutex to receive message to worker in a poisoned state.") // might fail if the mutex is in a poisoned state (e.g. other thread paniced while holding the lock)
                    .recv() // receive message from channel
                    .expect("Worker: sending side of channel to worker is down.") // might fail, e.g. thread on sending side of the channel might have shut down
                ;

                match message {
                    ToWorkerMessage::Task(task) => {
                        let task_result = task.run();

                        let wrapped_task_result = ToOrchestratorMessage::TaskResult(task_result);

                        // TODO: once the task is complete send the result to orchestrator

                        to_orchestrator_sender
                            .lock()
                            .expect("Worker: mutex to send message to orchestrator in a poisoned state.")
                            .send(wrapped_task_result)
                            .expect("Worker: receiving side of channel to orchestrator down.");
                    }
                    ToWorkerMessage::Terminate => {
                        break;
                    }
                }
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}
