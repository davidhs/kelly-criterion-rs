//! A worker is responsible for carrying out a task.

use crate::multi::task::{Task};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use crate::multi::orchestrator::ToOrchestratorMessage;

/// Messages that are intended for the worker.
pub enum ToWorkerMessage {
    /// A worker gets assigned a new task.
    Task(Task),
    /// A worker should terminate
    Terminate,
}


pub struct Worker {
    /// ID of worker.
    id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker that starts spinning right away and awaits new
    /// tasks to execute.
    pub fn new(id: usize, to_worker_receiver: Arc<Mutex<Receiver<ToWorkerMessage>>>, to_orchestrator_sender: Arc<Mutex<Sender<ToOrchestratorMessage>>>) -> Worker {
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
            id: id,
            thread: Some(thread),
        }
    }
}
