use std::{cell::RefCell, collections::{BTreeMap, BinaryHeap, VecDeque}, io, sync::{Mutex, MutexGuard, mpsc}};

pub type Error = Box<dyn std::error::Error + Send + 'static>;
pub fn new_error(msg: impl Into<String>) -> Error {
    Box::new(io::Error::new(io::ErrorKind::Other, msg.into()))
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct TimeoutQueueKey (
    /// The time at which the timeout should be executed
    std::time::Instant,
    /// The timeout ID (used to cancel the timeout if needed)
    u64,
);

pub enum TimeoutQueueItem {
    Timeout(Box<dyn FnOnce() + 'static>),
    Interval((std::time::Duration, Box<dyn FnMut(u64) + 'static>)),
}

pub struct EventLoopStruct {
    task_queue: VecDeque<Box<dyn FnOnce() + 'static>>,
    timeout_queue: BTreeMap<TimeoutQueueKey, TimeoutQueueItem>,
    timeout_counter: u64,
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
    pub fn with_running<R>(f: impl FnOnce(&mut EventLoopStruct) -> R) -> Result<R, Error> {
        INSTANCE.with_borrow_mut(|instance| {
            if let EventLoop::Running(e) = instance {
                return Ok(f(e));
            } else {
                return Err(new_error("Event loop is not running!"));
            }
        })
    }
    /// Start the event loop, running indefinitely until the program is terminated
    /// This function should be called once and it will block the thread it's called on
    /// panics if called while the event loop is already running or stopped
    pub fn start(first_task: impl FnOnce()) -> Result<(), Error> {
        // Initialize the event loop
        INSTANCE.with_borrow_mut(|instance| {
            if let EventLoop::Uninitialized = instance {
                *instance = EventLoop::Running(EventLoopStruct {
                    task_queue: VecDeque::new(),
                    timeout_queue: BTreeMap::new(),
                    timeout_counter: 0,
                });
                return Ok(());
            } else {
                return Err(new_error("Event loop is already running or stopped!"));
            }
        })?;

        // run the first task
        first_task();

        // run the event loop
        // println!("Event loop waiting for tasks...");
        loop {
            // Get the next task from the queue
            let maybe_task = EventLoop::with_running(|e| {
                e.task_queue.pop_front()
            })?;

            if let Some(task) = maybe_task {
                // Execute the task
                task();
            } else {
                // Now check and run a timeout (they're ordered)
                let now = std::time::Instant::now();
                let (timeout, has_next) = EventLoop::with_running(|e| {
                    let Some(mut entry) = e.timeout_queue.first_entry() else {
                        return (None, false);
                    };
                    if entry.key().0 > now {
                        return (None, true);
                    }

                    let key = entry.key().clone();
                    let item = entry.remove();
                    (Some((key, item)), true)
                })?;

                if let Some((timeout_key, timeout_item)) = timeout {
                    match timeout_item {
                        TimeoutQueueItem::Timeout(f) => f(),
                        TimeoutQueueItem::Interval((delay, mut f)) => {
                            f(timeout_key.1);
                            // Reschedule the same interval with the same key to allow cancellation
                            EventLoop::with_running(|e| {
                                e.timeout_queue.insert(
                                    TimeoutQueueKey(timeout_key.0 + delay, timeout_key.1), 
                                    TimeoutQueueItem::Interval((delay, f))
                                );
                            })?;
                        },
                    }
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
        F: FnOnce() + 'static,
    {
        EventLoop::with_running(|e| {
            e.task_queue.push_back(Box::new(f));
        })
    }

    /// Put the timeout on the queue ordered
    pub fn set_timeout<F>(f: F, ms: u64) -> Result<u64, Error> 
    where
        F: FnOnce() + 'static,
    {
        let timeout_time = std::time::Instant::now() + std::time::Duration::from_millis(ms);        
        EventLoop::with_running(|e| {
            let timeout_id = e.timeout_counter;
            e.timeout_counter += 1;

            e.timeout_queue.insert(
                TimeoutQueueKey(timeout_time, timeout_id), 
                TimeoutQueueItem::Timeout(Box::new(f))
            );
            timeout_id
        })
    }

    pub fn set_interval<F>(mut f: F, ms: u64) -> Result<u64, Error> 
    where
        F: FnMut(u64) + 'static,
    {
        let timeout_time = std::time::Instant::now() + std::time::Duration::from_millis(ms);        
        EventLoop::with_running(|e| {
            let timeout_id = e.timeout_counter;
            e.timeout_counter += 1;

            e.timeout_queue.insert(
                TimeoutQueueKey(timeout_time, timeout_id), 
                TimeoutQueueItem::Interval((std::time::Duration::from_millis(ms), Box::new(f)))
            );
            timeout_id
        })
    }

    pub fn clear_timeout(timeout_id: u64) -> Result<(), Error> {
        let f = move || {
            // Remove the timeout with the given ID from the queue, if it exists
            EventLoop::with_running(|e| {
                // iterate over the timeout queue and remove the first entry with the given ID
                let found_key = e.timeout_queue.keys().find(|key| key.1 == timeout_id).cloned();
                if let Some(key) = found_key {
                    e.timeout_queue.remove(&key);
                }
            }).unwrap();
        };
        // Schedules the clear_timeout to run on the next tick in the event loop to avoid issues with timeouts auto-canceling
        // push_front, so it runs before any other task in the next tick
        EventLoop::with_running(|e| {
            e.task_queue.push_front(Box::new(f));
        })
    }
}