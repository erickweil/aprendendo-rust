use std::{cell::RefCell, collections::{BinaryHeap, VecDeque}, io, sync::{Mutex, MutexGuard, mpsc}};

pub type Error = Box<dyn std::error::Error + Send + 'static>;

pub struct TimeoutQueueItem (std::time::Instant, Box<dyn FnOnce() -> Result<(), Error> + 'static>);

impl Eq for TimeoutQueueItem {}
impl PartialEq for TimeoutQueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Ord for TimeoutQueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We want the smallest timeout to be the first one, so we reverse the order
        other.0.cmp(&self.0)
    }
}
impl PartialOrd for TimeoutQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct EventLoopStruct {
    task_queue: VecDeque<Box<dyn FnOnce() -> Result<(), Error> + 'static>>,
    timeout_queue: BinaryHeap<TimeoutQueueItem>,
}

/// The event loop is responsible for managing the execution of asynchronous tasks (single threaded)
/// It receives tasks and executes them in order
pub enum EventLoop {
    Uninitialized,
    Running(EventLoopStruct),
    Stopped,
}

/// Singleton that allows only one instance of the event loop to exist
thread_local! {
    static INSTANCE: RefCell<EventLoop> = const { 
        RefCell::new(EventLoop::Uninitialized)
    };
}
impl EventLoop {
    pub fn with_running<R>(f: impl FnOnce(&mut EventLoopStruct) -> Result<R, Error>) -> Result<R, Error> {
        Ok(INSTANCE.with_borrow_mut(|instance| {
            if let EventLoop::Running(e) = instance {
                return f(e)
            } else {
                return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Event loop is not running!")))
            }
        })?)
    }
    /// Start the event loop, running indefinitely until the program is terminated
    /// This function should be called once and it will block the thread it's called on
    /// panics if called while the event loop is already running or stopped
    pub fn start(first_task: impl FnOnce() -> Result<(), Error>) -> Result<(), Error> {
        // Initialize the event loop
        INSTANCE.with_borrow_mut(|instance| {
            if let EventLoop::Uninitialized = instance {
                *instance = EventLoop::Running(EventLoopStruct {
                    task_queue: VecDeque::new(),
                    timeout_queue: BinaryHeap::new(),
                });
            } else {
                panic!("Event loop is already running or stopped!");
            }
        });

        // run the first task
        first_task()?;

        // run the event loop
        // println!("Event loop waiting for tasks...");
        loop {
            // Get the next task from the queue
            let maybe_task = EventLoop::with_running(|e| {
                Ok(e.task_queue.pop_front())
            })?;

            if let Some(task) = maybe_task {
                // Execute the task and if it returns an error, stop the event loop
                task()?;
            } else {
                // Now check and run a timeout (they're ordered)
                let now = std::time::Instant::now();
                let (timeout_fn, has_next) = EventLoop::with_running(|e| {
                    let Some(maybe_timeout_fn) = e.timeout_queue.peek() else {
                        return Ok((None, false));
                    };
                    if maybe_timeout_fn.0 > now {
                        return Ok((None, true));
                    }

                    Ok((e.timeout_queue.pop().map(|t| t.1), true))
                })?;

                if let Some(timeout_fn) = timeout_fn {
                    timeout_fn()?;
                } else if has_next {
                    // Some more tasks, sleep for a bit to avoid busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(4));
                } else {
                    // Nothing to do, stopping the event loop
                    EventLoop::stop();
                    
                    return Ok(());
                }
            }
        }
    }

    /// Clear all pending tasks and stop the event loop
    pub fn stop() {
        // println!("Stopping event loop...");
        INSTANCE.with_borrow_mut(|instance| {
            *instance = EventLoop::Stopped;
        });
    }

    /// Spawn a task to be executed by the event loop
    pub fn spawn<F>(f: F) -> Result<(), Error> 
    where
        F: FnOnce() -> Result<(), Error> + 'static,
    {
        EventLoop::with_running(|e| {
            e.task_queue.push_back(Box::new(f));
            Ok(())
        })
    }

    /// Put the timeout on the queue ordered
    pub fn set_timeout<F>(f: F, ms: u64) -> Result<(), Error> 
    where
        F: FnOnce() -> Result<(), Error> + 'static,
    {
        let timeout_time = std::time::Instant::now() + std::time::Duration::from_millis(ms);        
        EventLoop::with_running(|e| {
            e.timeout_queue.push(TimeoutQueueItem(timeout_time, Box::new(f)));
            Ok(())
        })
    }

    /// Naive setInterval, calls set_timeout recursively
    pub fn set_interval<F>(mut f: F, ms: u64) -> Result<(), Error> 
    where
        F: FnMut() -> Result<bool, Error> + 'static,
    {
        EventLoop::set_timeout(move || {
            let should_repeat = f()?;
            if should_repeat {
                EventLoop::set_interval(f, ms)?;
            }
            Ok(())
        }, ms)
    }
}