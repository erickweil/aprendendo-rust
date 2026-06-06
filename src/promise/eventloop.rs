use std::{cell::{OnceCell, RefCell}, collections::{BTreeMap, BinaryHeap, HashMap, VecDeque}, io, sync::{Mutex, MutexGuard, OnceLock, mpsc}, thread::JoinHandle, time::{Duration, Instant, SystemTime}};

pub type BoxedError = Box<dyn std::error::Error + Send + 'static>;
pub fn new_error(msg: impl Into<String>) -> BoxedError {
    Box::new(io::Error::new(io::ErrorKind::Other, msg.into()))
}

// https://github.com/rust-lang/rust/issues/71224
pub fn add_duration(time: Instant, duration: Duration) -> Instant {
    time.checked_add(duration).unwrap_or_else(|| {
        // Safe far future, std doesn't have Instant::MAX
        Instant::now() + Duration::from_secs(86400 * 365 * 30)
    })
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct TimeoutQueueKey (
    /// The time at which the timeout should be executed
    std::time::Instant,
    /// The timeout ID (used to cancel the timeout if needed)
    u64,
);

enum TimeoutQueueNext {
    Ready(Box<dyn FnOnce() -> Result<(), BoxedError> + 'static>),
    NotReady(Duration),
    Empty,
}

/// Unified structure that keeps BTreeMap and HashMap in sync automatically
/// Without this code grows and becomes error-prone with duplicated logic
struct TimeoutQueue {
    /// Optimized to get the next timeout and/or obtain a random timeout by its key
    queue: BTreeMap<TimeoutQueueKey, Box<dyn FnOnce() -> Result<(), BoxedError> + 'static>>,
    /// Optimized to cancel timeouts by their ID solely, mapping timeout ID to the BTreeMap key
    index: HashMap<u64, TimeoutQueueKey>,
    timeout_counter: u64,
}

impl TimeoutQueue {
    pub fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            index: HashMap::new(),
            timeout_counter: 0,
        }
    }

    pub fn create_timeout_key(&mut self, delay: Duration) -> TimeoutQueueKey {
        let timeout_id = self.timeout_counter;
        self.timeout_counter += 1;

        TimeoutQueueKey(add_duration(Instant::now(), delay), timeout_id)
    }

    pub fn insert(&mut self, key: TimeoutQueueKey, timeout_fn: Box<dyn FnOnce() -> Result<(), BoxedError> + 'static>) -> u64 {
        let timeout_id = key.1;
        self.index.insert(timeout_id, key.clone());
        self.queue.insert(key, timeout_fn);

        timeout_id
    }

    pub fn remove_by_id(&mut self, timeout_id: u64) -> Option<Box<dyn FnOnce() -> Result<(), BoxedError> + 'static>> {
        let key = self.index.remove(&timeout_id)?;
        self.queue.remove(&key)
    }

    /// Get the next timeout that is ready to be executed, if there is not any ready timeout, return the Duration to it
    pub fn pop_if_ready(&mut self) -> TimeoutQueueNext {
        let Some(entry) = self.queue.first_entry() else {
            return TimeoutQueueNext::Empty;
        };

        let now = std::time::Instant::now();
        let entry_key = entry.key();
        if entry_key.0 > now {
            return TimeoutQueueNext::NotReady(entry_key.0 - now);
        }

        // Remove from the ID map also
        self.index.remove(&entry_key.1);
        let item = entry.remove();
        TimeoutQueueNext::Ready(item)
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.index.clear();
        // don't reset counter
    }
}

pub struct EventLoop {
    task_queue: VecDeque<Box<dyn FnOnce() -> Result<(), BoxedError> + 'static>>,
    timeout_queue: TimeoutQueue,
    event_loop_rx: mpsc::Receiver<Box<dyn FnOnce() -> Result<(), BoxedError> + Send + 'static>>,
    is_running: bool,
}

/// Singleton that allows only one instance of the event loop to exist
thread_local! {
    static INSTANCE: OnceCell<RefCell<EventLoop>> = const { 
        OnceCell::new()
    };
}

/// A channel that allows tasks to be spawned from other threads to the event loop thread
static EVENT_LOOP_TX: OnceLock<mpsc::Sender<Box<dyn FnOnce() -> Result<(), BoxedError> + Send + 'static>>> = OnceLock::new();

impl EventLoop {
    /// Will initialize the EventLoop if it is not already
    /// Produce an error if it was already initialized in another thread
    fn with<R>(f: impl FnOnce(&mut EventLoop) -> R) -> Result<R, BoxedError> {
        INSTANCE.with(|instance| {
            let event_loop = if let Some(event_loop) = instance.get() {
                event_loop
            } else {
                // Create the channel for remote task spawning
                let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() -> Result<(), BoxedError> + Send + 'static>>();

                // Fails if another thread has already initialized the event loop and set the channel
                EVENT_LOOP_TX.set(tx).map_err(|_| new_error("Failed to set remote spawner channel! EventLoop was already initialized in another thread?"))?;
                
                instance.set(RefCell::new(EventLoop {
                    task_queue: VecDeque::new(),
                    timeout_queue: TimeoutQueue::new(),
                    event_loop_rx: rx,
                    is_running: false,
                })).ok();
                
                instance.get().unwrap()
            };

            Ok(f(&mut event_loop.borrow_mut()))
        })
    }

    /// this will panic if the event loop isn't running
    fn with_unchecked<R>(f: impl FnOnce(&mut EventLoop) -> R) -> R {
        INSTANCE.with(|instance| {
            let event_loop = instance.get().expect("EventLoop is not running!");
            f(&mut event_loop.borrow_mut())
        })
    }

    pub fn start<F>(first_task: F) -> Result<(), BoxedError>
    where 
        F: FnOnce() -> Result<(), BoxedError>,
    {
        // run the first task
        first_task()?;
        // run the event loop
        EventLoop::run_event_loop()
    }

    /// Run the event loop and block until there are no more tasks to execute and no more timeouts scheduled
    /// Only a single thread can ever run the event loop. Trying to run it from other threads will panic
    pub fn run_event_loop() -> Result<(), BoxedError> {
        // Get the event loop once using .with() to ensure it's initialized
        EventLoop::with(|e| {
            e.is_running = true;
        })?;

        let result = loop {
            match EventLoop::run_event_loop_tick() {
                Err(e) => {
                    // If an error is returned from a task, stop the event loop and return the error
                    break Err(e);
                },
                Ok(should_stop) if should_stop => {
                    // If the event loop tick indicates that we should stop (no more tasks or timeouts), break the loop and return Ok
                    break Ok(());
                },
                Ok(_) => {
                    // Otherwise, continue running the event loop
                },
            }
        };

        EventLoop::with_unchecked(|e| {
            e.is_running = false;
        });
        result
    }

    /// If it returns true or an error, the event loop will stop.
    /// Runs at most one task.
    /// Priority:
    /// 0. Tasks in the task queue (spawned with `EventLoop::spawn`)
    /// 1. Timeouts in the timeout queue (scheduled with `EventLoop::set_timeout` or `EventLoop::set_interval`) that are ready to run
    /// 2. Scheduling of remote tasks waiting in the channel (spawned with `EventLoop::spawn_remote`)
    fn run_event_loop_tick() -> Result<bool, BoxedError> {
        // 0. Get the next task from the queue
        let maybe_task = EventLoop::with_unchecked(|e| {
            e.task_queue.pop_front()
        });

        if let Some(task) = maybe_task {
            // Execute the task
            task()?;
            return Ok(false);
        }

        // 1. Now check and run a timeout (they're ordered)
        let next = EventLoop::with_unchecked(|e| {
            e.timeout_queue.pop_if_ready()
        });

        let time_next = match next {
            TimeoutQueueNext::Ready(timeout_fn) => {
                timeout_fn()?;
                return Ok(false);
            },
            TimeoutQueueNext::Empty => {
                // If there are no paused tasks and no timeouts scheduled, we can stop the event loop
                return Ok(true);
            }
            TimeoutQueueNext::NotReady(time_next) => time_next,
        };

        // 2. If there are no tasks or timeouts to run, wait for a remote task to be spawned or a timeout to be ready
        return EventLoop::with_unchecked(|e| {
            let maybe_remote_task = match e.event_loop_rx.recv_timeout(time_next) {
                Ok(task) => Some(task),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(e) => return Err(new_error(format!("Failed to receive remote task: {}", e))),
            };

            if let Some(remote_task) = maybe_remote_task {
                e.task_queue.push_back(remote_task);
            }
            return Ok(false)
        });
    }

    /// Clear all pending tasks to gracefully stop the event loop
    pub fn stop() -> Result<(), BoxedError> {
        EventLoop::with(|e| {
            e.task_queue.clear();
            e.timeout_queue.clear();
        })
    }

    /// Spawn a task to be executed by the event loop
    pub fn spawn<F>(f: F) -> Result<(), BoxedError> 
    where
        F: FnOnce() -> Result<(), BoxedError> + 'static,
    {
        EventLoop::with(|e| {
            e.task_queue.push_back(Box::new(f));
        })
    }

    /// It'll schedule a task in the event loop
    /// If the event loop is running it'll receive the task and execute it,
    /// If is not running a error is returned
    pub fn try_spawn_remote<F>(f: F) -> Result<(), BoxedError>
    where
        F: FnOnce() -> Result<(), BoxedError> + Send + 'static,
    {
        if let Some(tx) = EVENT_LOOP_TX.get() {
            tx.send(Box::new(f))
                .map_err(|e| new_error(format!("Failed to send remote task: {}", e)))?;
            Ok(())
        } else {
            Err(new_error("Remote spawner channel is not initialized!"))
        }
    }

    /// This is the only method that can be called safely from other threads. 
    /// It'll schedule a task in the event loop, but i'll do it in a new thread
    /// If the event loop is running it'll receive the task and execute it and the new thread will end immediately,
    /// If is not running it will start the event loop in the new thread and run it until completion
    /// (then after the event loop ends, the thread will end too and any future attempt to start a new EventLoop will fail)
    pub fn spawn_remote<F>(f: F) -> JoinHandle<Result<(), BoxedError>>
    where
        F: FnOnce() -> Result<(), BoxedError> + Send + 'static,
    {
        std::thread::spawn(move || {
            // Try to spawn a dummy task in the event loop, if it suceeds is because no event loop ever started in any thread
            if let Err(e) = EventLoop::spawn(|| { Ok(()) }) {
                // failed, so there is an event loop running in another thread
                // Send the task to the event loop to be executed
                EventLoop::try_spawn_remote(f)?;
                return Ok(());
            }

            // If the event loop is not running, start it with the given task as the first task
            EventLoop::start(f)
        })
    }

    /// Put the timeout on the queue ordered
    pub fn set_timeout<F>(f: F, delay: Duration) -> Result<u64, BoxedError> 
    where
        F: FnOnce() -> Result<(), BoxedError> + 'static,
    {
        EventLoop::with(|e| {
            let key = e.timeout_queue.create_timeout_key(delay);
            e.timeout_queue.insert(key, Box::new(f))
        })
    }

    pub fn set_interval<F>(mut f: F, delay: Duration) -> Result<u64, BoxedError> 
    where
        F: FnMut(u64) -> Result<(), BoxedError> + 'static,
    {
        fn interval_fn<F>(mut f: F, delay: Duration, key: TimeoutQueueKey) -> Result<(), BoxedError> 
        where
            F: FnMut(u64) -> Result<(), BoxedError> + 'static,
        {
            f(key.1)?;
            
            // Re-agenda com o MESMO timeout_id
            EventLoop::with_unchecked(|e| {
                let key = TimeoutQueueKey(add_duration(key.0, delay), key.1);
                e.timeout_queue.insert(key.clone(), Box::new(move || {
                    interval_fn(f, delay, key)
                }));
            });

            Ok(())
        }

        EventLoop::with(|e| {
            let key = e.timeout_queue.create_timeout_key(delay);
            e.timeout_queue.insert(key.clone(), Box::new(move || {
                interval_fn(f, delay, key)
            }))
        })
    }

    /// Re-schedule a timeout, putting it directly in the microtask queue
    pub fn wake_timeout(timeout_id: u64) -> Result<(), BoxedError> {
        EventLoop::with(|e| {
            // 1. find task and remove it
            let Some(task) = e.timeout_queue.remove_by_id(timeout_id) else {
                return Err(new_error(format!("No timeout found with id {}", timeout_id)));
            };

            // 2. push to task queue
            e.task_queue.push_back(task);

            Ok(())
        })?
    }

    pub fn clear_timeout(timeout_id: u64) -> Result<(), BoxedError> {
        let f = move || {
            EventLoop::with_unchecked(|e| {
                // find task and remove it, ignoring errors
                let _ = e.timeout_queue.remove_by_id(timeout_id);

                Ok(())
            })
        };
        // Schedules the clear_timeout to run on the next tick in the event loop to avoid issues with timeouts auto-canceling
        // push_front, so it runs before any other task in the next tick
        EventLoop::with(|e| {
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
    fn test_spawn() -> Result<(), BoxedError> {
        let counter = TestCounter::new();
        assert_eq!(counter.increment(), 1);

        let _counter = counter.clone();
        EventLoop::spawn_remote(move || {
            assert_eq!(_counter.increment(), 2);

            let __counter = _counter.clone();
            EventLoop::spawn(move || {
                assert_eq!(__counter.increment(), 4);
                Ok(())
            })?;

            let __counter = _counter.clone();
            EventLoop::spawn(move || {
                assert_eq!(__counter.increment(), 5);
                Ok(())
            })?;

            assert_eq!(_counter.increment(), 3);

            Ok(())
        }).join().unwrap()
    }

    #[test]
    fn test_timeout() -> Result<(), BoxedError> {
        EventLoop::spawn_remote(|| {
            let counter = TestCounter::new();

            let timeout_id = EventLoop::set_timeout(move || {
                panic!("This timeout should have been cleared!");
            }, Duration::from_millis(0))?;

            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 5);
                Ok(())
            }, Duration::from_millis(1))?;
            
            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 3);
                Ok(())
            }, Duration::from_millis(0))?;

            let _counter = counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(_counter.increment(), 4);
                Ok(())
            }, Duration::from_millis(0))?;

            let _counter = counter.clone();
            EventLoop::spawn(move || {
                assert_eq!(_counter.increment(), 2);
                Ok(())
            })?;

            assert_eq!(counter.increment(), 1);

            EventLoop::clear_timeout(timeout_id)?;
            Ok(())
        }).join().unwrap()
    }

    #[test]
    fn test_interval() -> Result<(), BoxedError> {
        let counter = TestCounter::new();

        let _counter = counter.clone();
        EventLoop::spawn_remote(move || {
            let __counter = _counter.clone();  
            EventLoop::set_interval(move |id| {
                let count = __counter.increment();
                if count == 5 {
                    EventLoop::clear_timeout(id)?;
                }
                Ok(())
            }, Duration::from_millis(1))?;

            assert_eq!(_counter.increment(), 1);
            Ok(())
        }).join().unwrap()
    }

    #[test]
    fn test_microtask_recursive() -> Result<(), BoxedError> {
        // This recursive spawning should not cause stack overflow
        // also it will only run set_timeout (Macrotask) after all the recursive spawns (Microtask)
        fn recursive_fn(counter: TestCounter) -> Result<(), BoxedError> {
            if counter.increment() > 100000 {
                return Ok(());
            }
            EventLoop::spawn(|| {
                recursive_fn(counter)
            })
        }

        let counter = TestCounter::new();

        let _counter = counter.clone();
        EventLoop::spawn_remote(move || {
            let __counter = _counter.clone();
            recursive_fn(__counter)?;

            let __counter = _counter.clone();
            EventLoop::set_timeout(move || {
                assert_eq!(__counter.increment(), 100002);
                Ok(())
            }, Duration::from_millis(0))?;

            Ok(())
        }).join().unwrap()
    }
}