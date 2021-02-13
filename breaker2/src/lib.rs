use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum BreakerError{
    PoolCreationError,
    WorkerCreationError,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

// Worker
struct Worker{
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker{
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Result<Self, BreakerError> {
        let handle = thread::spawn( move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    // thread::sleep(time::Duration::from_secs(5));
                    job();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                },
            }
        });

        Ok(Worker{ id, handle: Some(handle) })
        // } else {
        //     Err(BreakerError::WorkerCreationError)
        // }
    }
}

// Thread Pool
pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool{
    pub fn new(size: usize) -> Result<ThreadPool, BreakerError> {
        if size > 0 && size < 5 {
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver))?);
            }
            
            Ok(ThreadPool{ workers, sender })

        } else { 
            Err(BreakerError::PoolCreationError)
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}