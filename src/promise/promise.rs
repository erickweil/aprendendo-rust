use std::{cell::RefCell, convert::Infallible, fmt::Debug, io, mem, rc::Rc, result, sync::{Arc, Mutex}, time::Duration};
use crate::promise::{BoxedError, EventLoop, new_error};
//use std::ops::{FromResidual};

pub struct PromiseResolveRejectSend<T> {
    tx: oneshot::Sender<Result<T, BoxedError>>,
    task_id: u64,
}

impl<T: 'static + Send> PromiseResolveRejectSend<T> {
    fn _handle(self, value: Result<T, BoxedError>) {
        // Send the result to the waiting task, ignoring errors (e.g., if the receiver was dropped)
        self.tx.send(value).ok();

        let task_id = self.task_id;
        EventLoop::try_spawn_remote(move || {
            EventLoop::wake_timeout(task_id)
        }).unwrap();
    }

    pub fn resolve(self, value: T) {
        self._handle(Ok(value));
    }

    pub fn reject<E>(self, err: E) 
    where E: std::error::Error + Send + 'static,
    {
        self._handle(Err(Box::new(err)));
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
    pub fn resolve(self, value: T) {
        self.state.borrow_mut().on_settle(Ok(value));
    }

    pub fn reject<E>(self, err: E)
    where E: std::error::Error + Send + 'static,
    {
        self.state.borrow_mut().on_settle(Err(Box::new(err)));
    }
}

impl<T: Send + 'static> PromiseResolveReject<T> {
    pub fn into_send(self) -> PromiseResolveRejectSend<T> {
        let (tx, rx) = oneshot::channel::<Result<T,BoxedError>>();

        // paused task that when called handle the promise
        let task_id = EventLoop::set_timeout(move || {
            let result = rx.try_recv().ok();
            if let Some(result) = result {
                self.state.borrow_mut().on_settle(result);
            } else {
                self.state.borrow_mut().on_settle(Err(new_error("Nenhum valor ou erro foi definido para a promessa")));
            }
            Ok(())
        }, Duration::MAX).unwrap();

        PromiseResolveRejectSend {
            tx: tx,
            task_id,
        }
    }
}

// Se a promise for Dropada e é um PendingCatch com erro, lançar unhandled rejection
impl <T> Drop for PromiseResolveReject<T> {
    fn drop(&mut self) {
        let state = self.state.borrow();
        if let PromiseState::PendingCatch(err) = &*state {
            // Gracefully handle unhandled promise rejections by scheduling a task that throws an Err in the event loop
            let err = new_error(format!("Unhandled Promise rejection: {}", err));
            EventLoop::spawn(move || {
                Err(err)
            }).ok();
        }
    }
}

enum PromiseState<T> {
    // 0. nem then/catch/finally nem resolve/reject aconteceu -> apenas guardar o estado de pending
    Pending,
    // 1. then/catch/finally acontece antes de resolve/reject -> guardar callback e spawn da task quando chegar o resultado
    PendingResolve(Box<dyn FnOnce(Result<T, BoxedError>)>),
    // 2. resolve acontece antes de then/finally -> guardar valor, e quando daí depois spawn task já com o resultado
    PendingThen(T),
    // 3. reject acontece antes de catch/finally -> guardar valor, e quando daí depois spawn task já com o resultado
    PendingCatch(BoxedError),
    // 4. Já recebeu resultado e já executou e já foi movida, nada mais a fazer
    Settled,
}


impl<T> Debug for PromiseState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromiseState::Pending => write!(f, "Pending"),
            PromiseState::PendingResolve(_) => write!(f, "PendingResolve"),
            PromiseState::PendingThen(_) => write!(f, "PendingThen(?)"),
            PromiseState::PendingCatch(e) => write!(f, "PendingCatch({})", e),
            PromiseState::Settled => write!(f, "Settled"),
        }
    }
}

impl<T: 'static> PromiseState<T> {
    fn on_callback<C>(&mut self, callback: C)
    where 
        C: FnOnce(Result<T, BoxedError>) + 'static
    {
        match self {
            PromiseState::Pending => {
                *self = PromiseState::PendingResolve(Box::new(callback));
            },
            PromiseState::PendingThen(_) => {
                let PromiseState::PendingThen(value) = mem::replace(self,PromiseState::Settled) else { unreachable!() };
                // Investigar se deve chamar via EventLoop ou pode chamar direto
                EventLoop::spawn(move || {
                    callback(Ok(value));
                    Ok(())
                }).unwrap();
            },
            PromiseState::PendingCatch(_) => {
                let PromiseState::PendingCatch(err) = mem::replace(self,PromiseState::Settled) else { unreachable!() };
                // Investigar se deve chamar via EventLoop ou pode chamar direto
                EventLoop::spawn(move || {
                    callback(Err(err));
                    Ok(())
                }).unwrap();
            },
            PromiseState::PendingResolve(_) => {
                panic!("Promise already has a pending callback");
            },
            PromiseState::Settled => {
                panic!("Promise already settled");
            }
        }
    }

    fn on_settle(&mut self, value: Result<T, BoxedError>) {
        match self {
            PromiseState::Pending => {
                *self = match value {
                    Ok(v) => PromiseState::PendingThen(v),
                    Err(e) => PromiseState::PendingCatch(e),
                };
            },
            PromiseState::PendingResolve(_) => {
                let PromiseState::PendingResolve(callback) = mem::replace(self,PromiseState::Settled) else { unreachable!() };
                // Investigar se deve chamar via EventLoop ou pode chamar direto
                EventLoop::spawn(move || {
                    callback(value);
                    Ok(())
                }).unwrap();
            },
            PromiseState::PendingCatch(_) => {
                panic!("Promise already has a pending error");
            },
            PromiseState::PendingThen(_) => {
                panic!("Promise already has a result");
            },
            PromiseState::Settled => {
                panic!("Promise already settled");
            }
        }
    }
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
        let state = Rc::new(RefCell::new(PromiseState::PendingThen(value)));
        Self { state }
    }

    pub fn reject<E>(err: E) -> Self 
    where E: std::error::Error + Send + 'static,
    {
        let state = Rc::new(RefCell::new((PromiseState::PendingCatch(Box::new(err)))));
        Self { state }
    }

    pub fn reject_boxed(err: BoxedError) -> Self
    {
        let state = Rc::new(RefCell::new((PromiseState::PendingCatch(err))));
        Self { state }
    }

    fn handle_result<K: 'static>(
        self,
        f: impl FnOnce(Result<T, BoxedError>) -> Result<K, BoxedError> + 'static
    ) -> Promise<K> {
        let return_state = Rc::new(RefCell::new(PromiseState::Pending));

        let callback_state = return_state.clone();
        self.state.borrow_mut().on_callback(move |result: Result<T, BoxedError>| {
            let result_k = f(result);
            callback_state.borrow_mut().on_settle(result_k);         
        });

        return Promise { state: return_state };
    }

    /// It's like .then() but the callback can return a Result, where Ok will continue the chain and Err will jump to the nearest catch
    pub fn map<K: 'static, E>(
        self,
        f: impl FnOnce(T) -> Result<K, E> + 'static
    ) -> Promise<K> 
    where E: std::error::Error + Send + 'static,
    {
        self.handle_result(move |result| {
            match result {
                Ok(value) => f(value).map_err(|e| Box::new(e) as BoxedError),
                Err(err) => Err(err),
            }
        })
    }

    /// It's like .catch() but the callback can return a Result, where Ok will continue the chain and Err will jump to the nearest catch
    pub fn map_err<K: 'static, E>(
        self,
        f: impl FnOnce(BoxedError) -> Result<T, E> + 'static
    ) -> Promise<T> 
    where E: std::error::Error + Send + 'static,
    {
        self.handle_result(move |result| {
            match result {
                Ok(value) => Ok(value),
                Err(err) => f(err).map_err(|e| Box::new(e) as BoxedError),
            }
        })
    }

    fn handle<K: 'static>(
        self,
        f: impl FnOnce(Result<T, BoxedError>) -> Promise<K> + 'static
    ) -> Promise<K> {
        let return_state = Rc::new(RefCell::new(PromiseState::Pending));

        let callback_state = return_state.clone();
        self.state.borrow_mut().on_callback(move |result: Result<T, BoxedError>| {
            let promise_k = f(result);

            promise_k.state.borrow_mut().on_callback(move |result_k| {
                callback_state.borrow_mut().on_settle(result_k);
            });
        });

        return Promise { state: return_state };
    }

    pub fn then<K: 'static>( self, f: impl FnOnce(T) -> Promise<K> + 'static) -> Promise<K> {
        self.handle(move |result| {
            match result {
                Ok(value) => f(value),
                Err(err) => Promise::reject_boxed(err),
            }
        })
    }

    pub fn catch(self, f: impl FnOnce(BoxedError) -> Promise<T> + 'static) -> Promise<T> {
        self.handle(move |result| {
            match result {
                Ok(value) => Promise::resolve(value),
                Err(err) => f(err),
            }
        })
    }

    pub fn finally(self, f: impl FnOnce() + 'static) -> Self {
        self.handle_result(move |result| {
            f();
            result
        })
    }
}

/// Permite converter um Result em uma Promise, onde Ok se torna resolve e Err se torna reject
impl<T: 'static, E: std::error::Error + Send + 'static> From<Result<T, E>> for Promise<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(v)  => Promise::resolve(v),
            Err(e) => Promise::reject(Box::new(e)),
        }
    }
}

/*
nightly
/// Permite usar o operador `?` dentro do .then(), convertendo Err em reject da Promise
impl<T: 'static, E: std::error::Error + Send + 'static> FromResidual<Result<Infallible, E>> for Promise<T> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(e) => Promise::reject(Box::new(e)),
            _ => unreachable!(),
        }
    }
}*/

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::*;

    fn delay(ms: u64) -> Promise<()> {
        Promise::new(move |e| {
            EventLoop::set_timeout(move || {
                e.resolve(());
                Ok(())
            }, Duration::from_millis(ms)).unwrap();
        })
    }

    #[test]
    fn test_promise() -> Result<(), BoxedError> {
        EventLoop::spawn_remote(move || {
            let mut finally = Rc::new(RefCell::new(false));

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

            Ok(())
        }).join().unwrap()
    }

    #[test]
    fn test_delay() -> Result<(), BoxedError> {
        EventLoop::spawn_remote(move || {
            let start = std::time::Instant::now();
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
            }).finally(move || {
                let elapsed = start.elapsed();
                println!("Elapsed: {:?}", elapsed);
                assert!(elapsed >= std::time::Duration::from_millis(5));
                assert!(elapsed < std::time::Duration::from_millis(50));
            });
        
            Ok(())
        }).join().unwrap()
    }

    #[test]
    fn test_chaining() -> Result<(), BoxedError> {
        EventLoop::spawn_remote(move || {
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

            Ok(())
        }).join().unwrap()
    }
}