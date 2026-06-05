use std::{cell::RefCell, io, mem, rc::Rc, result, sync::{Arc, Mutex}};

use crate::promise::{Error, EventLoop, new_error};

pub struct PromiseResolveRejectSend<T> {
    tx: oneshot::Sender<Result<T, Error>>,
    task_id: u64,
}

impl<T: 'static + Send> PromiseResolveRejectSend<T> {
    fn _handle(self, value: Result<T, Error>) {
        // Send the result to the waiting task, ignoring errors (e.g., if the receiver was dropped)
        self.tx.send(value).ok();

        let task_id = self.task_id;
        EventLoop::spawn_remote(move || {
            EventLoop::resume_paused(task_id).unwrap();
        });
    }

    pub fn resolve(self, value: T) {
        self._handle(Ok(value));
    }

    pub fn reject(self, err: Error) {
        self._handle(Err(err));
    }
}

// drop?
// impl<T> Drop for PromiseResolveRejectSend<T> {
//     fn drop(&mut self) {
//         // If the PromiseResolveRejectSend is dropped without being resolved or rejected, we should clean up the paused task
//     }
// }


pub struct PromiseResolveReject<T> {
    state: Rc<RefCell<PromiseState<T>>>
}

impl<T: 'static> PromiseResolveReject<T> {
    fn _handle(self, value: Result<T, Error>) {
        let mut state = self.state.borrow_mut();
        if let PromiseState::Pending = &*state {
            mem::replace(
                &mut *state, 
                PromiseState::PendingThen(value)
            );
        } else if let PromiseState::PendingResolve(_) = &*state {
            let PromiseState::PendingResolve(callback) = mem::replace(
                &mut *state, 
                if value.is_ok() { PromiseState::Fulfilled } else { PromiseState::Rejected }
            ) else { unreachable!() };
            // Investigar se deve chamar via EventLoop ou pode chamar direto
            EventLoop::spawn(move || {
                callback(value);
            }).unwrap();
        } else if let PromiseState::PendingThen(_) = &*state {
            panic!("Promise already has a result");
        } else {
            panic!("Promise already settled");
        }
    }

    pub fn resolve(self, value: T) {
        self._handle(Ok(value));
    }

    pub fn reject(self, err: Error) {
        self._handle(Err(err));
    }
}

impl<T: Send + 'static> PromiseResolveReject<T> {
    pub fn into_send(self) -> PromiseResolveRejectSend<T> {
        let (tx, rx) = oneshot::channel::<Result<T,Error>>();

        // paused task that when called handle the promise
        let task_id = EventLoop::set_paused(move || {
            let result = rx.try_recv().ok();
            if let Some(result) = result {
                self._handle(result);
            } else {
                self.reject(new_error("Nenhum valor ou erro foi definido para a promessa"));
            }
        }).unwrap();

        PromiseResolveRejectSend {
            tx: tx,
            task_id,
        }
    }
}

enum PromiseState<T> {
    // 0. nem then/catch/finally nem resolve/reject aconteceu -> apenas guardar o estado de pending
    Pending,
    // 1. then/catch/finally acontece antes de resolve/reject -> guardar callback e spawn da task quando chegar o resultado
    PendingResolve(Box<dyn FnOnce(Result<T, Error>)>),
    // 2. resolve/reject acontece antes de then/catch/finally -> guardar valor, e quando daí depois spawn task já com o resultado
    PendingThen(Result<T, Error>),
    // Settled states
    Fulfilled,
    Rejected,
}

pub struct Promise<T> {
    state: Rc<RefCell<PromiseState<T>>>,
}

impl<T: 'static> Promise<T> {
    pub fn new<C>(run: C) -> Self 
    where 
        C: FnOnce(PromiseResolveReject<T>) + 'static,
    {
        let state = Rc::new(RefCell::new(PromiseState::Pending));

        run(PromiseResolveReject { state: state.clone() });

        Self { state }
    }

    pub fn resolve(value: T) -> Self {
        let state = Rc::new(RefCell::new(PromiseState::PendingThen(Ok(value))));
        Self { state }
    }

    pub fn reject(err: Error) -> Self {
        let state = Rc::new(RefCell::new(PromiseState::PendingThen(Err(err))));
        Self { state }
    }

    fn handle<K: 'static>(
        self,
        f: impl FnOnce(Result<T, Error>) -> Promise<K> + 'static
    ) -> Promise<K> {
        let return_state = Rc::new(RefCell::new(PromiseState::<K>::Pending));

        let resolve_reject = PromiseResolveReject { state: return_state.clone() };
        let handle_callback = move |result: Result<T, Error>| {
            let promise_k = f(result);
            promise_k.pipe_into(resolve_reject);
        };

        let mut state = self.state.borrow_mut();
        if let PromiseState::Pending = &*state {
            mem::replace(
                &mut *state,
                PromiseState::PendingResolve(Box::new(handle_callback))
            );
        } else if let PromiseState::PendingThen(_value) = &*state {
            let is_ok = _value.is_ok();

            let PromiseState::PendingThen(value) = mem::replace(
                &mut *state,
                if is_ok { PromiseState::Fulfilled } else { PromiseState::Rejected }
            ) else { unreachable!() };

            // Investigar se deve chamar via EventLoop ou pode chamar direto
            EventLoop::spawn(move || {
                handle_callback(value);
            }).unwrap();
        } else if let PromiseState::PendingResolve(_) = &*state {
            panic!("Promise already has a pending callback");
        } else {
            panic!("Promise already settled");
        }

        return Promise { state: return_state };
    }

    pub fn then<K: 'static>( self, f: impl FnOnce(T) -> Promise<K> + 'static) -> Promise<K> {
        self.handle(move |result| {
            match result {
                Ok(value) => f(value),
                Err(err) => Promise::reject(err),
            }
        })
    }

    pub fn catch(self, f: impl FnOnce(Error) -> Promise<T> + 'static) -> Promise<T> {
        self.handle(move |result| {
            match result {
                Ok(value) => Promise::resolve(value),
                Err(err) => f(err),
            }
        })
    }

    pub fn finally(self, f: impl FnOnce() + 'static) -> Self {
        self.handle(move |result| {
            f();
            match result {
                Ok(value) => Promise::resolve(value),
                Err(err) => Promise::reject(err),
            }
        })
    }

    /// Permite uma promessa ser "encadeada" em outra
    fn pipe_into(self, state: PromiseResolveReject<T>) {
        let handle_callback = move |result: Result<T, Error>| {
            state._handle(result);
        };

        let mut state = self.state.borrow_mut();
        if let PromiseState::Pending = &*state {
            mem::replace(
                &mut *state,
                PromiseState::PendingResolve(Box::new(handle_callback))
            );
        } else if let PromiseState::PendingThen(_value) = &*state {
            let is_ok = _value.is_ok();

            let PromiseState::PendingThen(value) = mem::replace(
                &mut *state,
                if is_ok { PromiseState::Fulfilled } else { PromiseState::Rejected }
            ) else { unreachable!() };

            // Investigar se deve chamar via EventLoop ou pode chamar direto
            handle_callback(value);
        } else if let PromiseState::PendingResolve(_) = &*state {
            panic!("Promise already has a pending callback");
        } else {
            panic!("Promise already settled");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delay(ms: u64) -> Promise<()> {
        Promise::new(move |e| {
            EventLoop::set_timeout(move || {
                e.resolve(());
            }, ms).unwrap();
        })
    }

    #[test]
    fn test_promise() {
        let mut finally = Rc::new(RefCell::new(false));
        EventLoop::start(|| {
            let finally_clone = finally.clone();
            Promise::new(|e| {
                e.resolve("OK A B C".to_string());
            }).then(|str| {
                assert_eq!(&str, "OK A B C");
                Promise::resolve(str.split(" ").map(|s| s.to_string()).collect::<Vec<String>>())
            }).then(|vec| {
                assert_eq!(vec, vec!["OK", "A", "B", "C"]);
                Promise::resolve(())
            }).catch(|e| {
                panic!("Não deveria cair no catch");
            }).finally(move || {
                *finally_clone.borrow_mut() = true;
            });
        }).unwrap();

        assert_eq!(*finally.borrow(), true);
    }

    #[test]
    fn test_delay() {
        let start = std::time::Instant::now();
        EventLoop::start(|| {
            Promise::resolve(()).then(|_| {
                delay(1)
            }).then(|_| {
                delay(1)
            }).then(|_| {
                delay(1)
            }).then(|_| {
                delay(1)
            }).then(|_| {
                delay(1)
            });
        }).unwrap();

        let elapsed = start.elapsed();
        println!("Elapsed: {:?}", elapsed);
        assert!(elapsed >= std::time::Duration::from_millis(5));
        assert!(elapsed < std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_chaining() {
        EventLoop::start(|| {
            let mut promise = Promise::resolve(0);
            for i in 0..10000 {
                promise = promise.then(move |counter| {
                    assert_eq!(counter, i);
                    Promise::resolve(counter + 1)
                });
            }

            promise.then(|counter| {
                println!("Counter final: {}", counter);
                assert_eq!(counter, 10000);
                Promise::resolve(())
            });
        }).unwrap();
    }
}