//! Thread pool for concurrent connection handling.
//!
//! Implements a fixed-size worker thread pool that distributes incoming
//! connections across threads using a channel-based job queue.

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A pool of worker threads for handling concurrent tasks.
///
/// # Example
/// ```no_run
/// use mini_http::thread_pool::ThreadPool;
///
/// let pool = ThreadPool::new(4);
/// for i in 0..8 {
///     pool.execute(move || {
///         println!("Task {} executed on thread {:?}", i, std::thread::current().id());
///     });
/// }
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new thread pool with the given number of worker threads.
    ///
    /// # Panics
    /// Panics if `size` is 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "ThreadPool size must be at least 1");

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Execute a job on a worker thread.
    ///
    /// The job will be queued and picked up by the next available worker.
    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(ref sender) = self.sender {
            sender.send(Box::new(job)).expect("Failed to send job to thread pool");
        }
    }

    /// Returns the number of worker threads.
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to signal workers to shut down
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::Builder::new()
            .name(format!("mini_http-worker-{}", id))
            .spawn(move || loop {
                let message = {
                    let lock = receiver.lock().expect("Worker mutex poisoned");
                    lock.recv()
                };

                match message {
                    Ok(job) => job(),
                    Err(_) => break, // Channel closed, shut down
                }
            })
            .expect("Failed to spawn worker thread");

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}
