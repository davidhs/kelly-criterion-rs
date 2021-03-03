use std::{collections::VecDeque, sync::mpsc::{Receiver, Sender, TryRecvError, channel}};
use crate::multi::task::{TaskResult, Task};
use std::sync::Arc;
use std::sync::Mutex;
use crate::multi::worker::{ToWorkerMessage, Worker};



pub enum ToOrchestratorMessage {
    TaskResult(TaskResult),
}

/// NOTE: only ever interact with the workers through the worker manager, never
/// the workers directly.
pub struct Orchestrator {
    workers: Vec<Worker>,
    tasks: VecDeque<Task>,
    task_results: Vec<TaskResult>,
    
    to_orchestrator_receiver: Receiver<ToOrchestratorMessage>,
    nr_of_available_workers: usize,
    nr_of_workers: usize,
    to_worker_sender: Sender<ToWorkerMessage>,
}

impl Orchestrator {
    
    pub fn new(tasks: VecDeque<Task>) -> Orchestrator {
        let nr_of_cpus = num_cpus::get();
        
        // Send a task from a single orchestrator to multiple workers.
        let (
            to_worker_sender,
            to_worker_receiver,
        ) = channel::<ToWorkerMessage>();
        
        // Send a task result from multiple works to a single orchestrator
        let (
            to_orchestrator_sender,
            to_orchestrator_receiver,
        ) = channel::<ToOrchestratorMessage>();
        
        // All workers ...
        let to_worker_receiver = Arc::new(Mutex::new(to_worker_receiver));
        let to_orchestrator_sender = Arc::new(Mutex::new(to_orchestrator_sender));
        
        let mut workers = Vec::with_capacity(nr_of_cpus);
        
        for id in 0..nr_of_cpus {
            let worker = Worker::new(
                id, 
                Arc::clone(&to_worker_receiver),
                Arc::clone(&to_orchestrator_sender),
            );
            workers.push(worker);
        }
        
        Orchestrator {
            workers,
            tasks,
            to_orchestrator_receiver: to_orchestrator_receiver,
            nr_of_available_workers: nr_of_cpus,
            nr_of_workers: nr_of_cpus,
            to_worker_sender: to_worker_sender,
            task_results: Vec::new(),
        }
    }
    
    pub fn run(&mut self) -> () {
        // Run task and task result loop.
        loop {
            // Check if we're done.
            if self.tasks.is_empty() && self.all_available() {
                break;
            }
            
            while !self.tasks.is_empty() {
                
                if self.none_available() {
                    // Stop this loop and wait on receiving a message from a worker
                    // to free a worker.
                    break;
                }
                
                // Dequeue task from queue.
                let task = self.tasks.pop_front().expect("");
                
                self.to_worker_sender.send(ToWorkerMessage::Task(task)).expect("");
                self.nr_of_available_workers -= 1;
            }
            
            // TODO: block on receiving message from worker
            let mut msg = self.to_orchestrator_receiver.recv().expect("");
            self.nr_of_available_workers += 1;
            
            match msg {
                ToOrchestratorMessage::TaskResult(task_result) => {
                    self.task_results.push(task_result);
                }
            }
            
            loop {
                
                // NOTE: once a message is received from a worker then that worker is
                // freed and available.
                
                // TODO: update available workers queue
                
                // TODO: Handle message (message_to_handle)
                
                // TODO: check to see if there are more messages from the worker, if
                // so handle them, until there are no more messages to handle.
                
                match self.to_orchestrator_receiver.try_recv() {
                    Ok(message) => {
                        // TODO: does this free the worker?
                        msg = message;
                        
                        match msg {
                            ToOrchestratorMessage::TaskResult(task_result) => {
                                self.task_results.push(task_result);
                            }
                        }
                        
                        self.nr_of_available_workers += 1;
                    }
                    Err(err) => {
                        match err {
                            TryRecvError::Empty => {
                                // Stop looping.
                                break;
                            },
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

        for _ in &self.workers {
            self.to_worker_sender.send(ToWorkerMessage::Terminate).unwrap();
        }

        // println!("Shutting down all workers.");

        for worker in &mut self.workers {
            // println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
        
        self.task_results.sort_by(|a, b| {
            a.id.partial_cmp(&b.id).unwrap()
        });
        
        // TODO: use typestate pattern (?)
        
        // TODO: dispose of wall workers
        
        // TODO: return result or display result
        
        // todo!();
    }
    
    pub fn get_results(self) -> Vec<TaskResult> {
        // TODO: sort task results
        self.task_results
    }
    
    pub fn all_available(&self) -> bool {
        self.nr_of_workers == self.nr_of_available_workers
    }
    
    // No worker available
    pub fn none_available(&self) -> bool {
        self.nr_of_available_workers == 0
    }
}