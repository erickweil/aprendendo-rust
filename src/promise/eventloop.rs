use std::{cell::RefCell, collections::{BTreeMap, BinaryHeap, HashMap, VecDeque}, io, sync::{Mutex, MutexGuard, OnceLock, mpsc}, time::{Duration, Instant}};

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
    paused_tasks: HashMap<u64, Box<dyn FnOnce() + 'static>>,
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

/// A channel that allows tasks to be spawned from other threads to the event loop thread
static REMOTE_SPAWNER: OnceLock<mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>> = OnceLock::new();

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
                    paused_tasks: HashMap::new(),
                    timeout_counter: 0,
                });
                return Ok(());
            } else {
                return Err(new_error("Event loop is already running or stopped!"));
            }
        })?;

        // Initialize the remote spawner channel
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>();
        REMOTE_SPAWNER.set(tx).map_err(|_| new_error("Failed to set remote spawner channel!"))?;

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
                let (timeout, time_next) = EventLoop::with_running(|e| {
                    let Some(mut entry) = e.timeout_queue.first_entry() else {
                        return (None, None);
                    };
                    if entry.key().0 > now {
                        return (None, Some(entry.key().0 - now));
                    }

                    let key = entry.key().clone();
                    let item = entry.remove();
                    (Some((key, item)), None)
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
                } else if let Some(time_next) = time_next {
                    // Wait until the next timer or until a remote task is spawned, whichever comes first
                    if let Ok(remote_task) = rx.recv_timeout(time_next) {
                        EventLoop::with_running(|e| {
                            e.task_queue.push_back(remote_task);
                        })?;
                    }
                } else {
                    // Check paused tasks 
                    let has_paused_tasks = EventLoop::with_running(|e| {
                        !e.paused_tasks.is_empty()
                    })?;

                    if has_paused_tasks {
                        // If there are paused tasks, block waiting for a remote task to resume them
                        if let Ok(remote_task) = rx.recv() {
                            EventLoop::with_running(|e| {
                                e.task_queue.push_back(remote_task);
                            })?;
                        }
                    } else {
                        // Nothing to do, stopping the event loop
                        INSTANCE.with_borrow_mut(|instance| {
                            *instance = EventLoop::Stopped;
                        });
                        
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Clear all pending tasks to gracefully stop the event loop
    pub fn stop() {
        // println!("Stopping event loop...");
        EventLoop::with_running(|e| {
            e.task_queue.clear();
            e.timeout_queue.clear();
        }).unwrap();
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

    pub fn spawn_remote<F>(f: F) -> Result<(), Error> 
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(tx) = REMOTE_SPAWNER.get() {
            tx.send(Box::new(f))
                .map_err(|e| new_error(format!("Failed to send remote task: {}", e)))?;
            Ok(())
        } else {
            Err(new_error("Remote spawner channel is not initialized!"))
        }
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

    /// Set a paused task that can be executed later with `EventLoop::resume_paused`
    pub fn set_paused<F>(f: F) -> Result<u64, Error> 
    where
        F: FnOnce() + 'static,
    {
        EventLoop::with_running(|e| {
            let timeout_id = e.timeout_counter;
            e.timeout_counter += 1;

            e.paused_tasks.insert(timeout_id, Box::new(f));
            timeout_id
        })
    }

    /// Resume a paused task with the given ID, putting it back on the task queue
    pub fn resume_paused(timeout_id: u64) -> Result<(), Error> {
        EventLoop::with_running(|e| {
            if let Some(task) = e.paused_tasks.remove(&timeout_id) {
                e.task_queue.push_back(task);
            }
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;

    #[derive(Clone, Debug)]
    struct TestCounter {
        count: Arc<Mutex<u64>>,
    }

    impl TestCounter {
        fn new() -> Self {
            Self { count: Arc::new(Mutex::new(0)) }
        }

        fn increment(&self) -> u64 {
            let mut count = self.count.lock().unwrap();
            *count += 1;
            *count
        }
    }

    impl PartialEq for TestCounter {
        fn eq(&self, other: &Self) -> bool {
            *self.count.lock().unwrap() == *other.count.lock().unwrap()
        }
    }


    #[test]
    fn test_spawn() {
        let counter = TestCounter::new();
        assert_eq!(counter.increment(), 1);

        let _counter = counter.clone();
        EventLoop::start(move || {
            assert_eq!(_counter.increment(), 2);

            let __counter = _counter.clone();
            EventLoop::spawn(move || {
                assert_eq!(__counter.increment(), 4);
            }).unwrap();

            let __counter = _counter.clone();
            EventLoop::spawn(move || {
                assert_eq!(__counter.increment(), 5);
            }).unwrap();

            assert_eq!(_counter.increment(), 3);
        }).unwrap();

        assert_eq!(counter.increment(), 6);
    }

    #[test]
    fn test_timeout() {
        EventLoop::start(|| {
            let counter = TestCounter::new();

            let timeout_id = EventLoop::set_timeout(move || {
                panic!("This timeout should have been cleared!");
            }, 0).unwrap();

            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 5);
            }, 1).unwrap();
            
            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 3);
            }, 0).unwrap();

            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 4);
            }, 0).unwrap();

            let _counter = counter.clone();
            EventLoop::spawn(move || {
                assert_eq!(_counter.increment(), 2);
            }).unwrap();

            assert_eq!(counter.increment(), 1);

            EventLoop::clear_timeout(timeout_id).unwrap();
        }).unwrap();
    }

    #[test]
    fn test_interval() {
        let counter = TestCounter::new();

        EventLoop::start(|| {
            let _counter = counter.clone();  
            EventLoop::set_interval(move |id| {
                let count = _counter.increment();
                if count == 5 {
                    EventLoop::clear_timeout(id).unwrap();
                }
            }, 1).unwrap();

            assert_eq!(counter.increment(), 1);
        }).unwrap();

        assert_eq!(counter.increment(), 6);
    }

    #[test]
    fn test_microtask_recursive() {
        // This recursive spawning should not cause stack overflow
        // also it will only run set_timeout (Macrotask) after all the recursive spawns (Microtask)
        fn recursive_fn(counter: TestCounter) {
            if counter.increment() > 100000 {
                return;
            }
            EventLoop::spawn(|| {
                recursive_fn(counter);
            }).unwrap();
        }

        let counter = TestCounter::new();
        EventLoop::start(|| {
            let _counter = counter.clone();
            recursive_fn(_counter);

            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 100002);
            }, 0);
        }).unwrap();

        assert_eq!(counter.increment(), 100003);
    }
}