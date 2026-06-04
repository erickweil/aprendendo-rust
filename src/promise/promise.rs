use std::{io, result, sync::{Arc, Mutex}};

use crate::promise::EventLoop;

pub type Error = Box<dyn std::error::Error + Send + 'static>;

pub struct PromiseResolveReject<T> {
    tx: oneshot::Sender<Result<T, Error>>
}

impl<T> PromiseResolveReject<T> {
    pub fn resolve(self, value: T) {
        self.tx.send(Ok(value)).ok();
    }

    pub fn reject(self, err: Error) {
        self.tx.send(Err(err)).ok();
    }
}

/// Promessa com comportamento similar ao JavaScript, singlethreaded rodando no EventLoop (Deve estar dentro de EventLoop::start)
/// A promessa inicia o processamento IMEDIATAMENTE na thread do event loop.
/// Não existe resolve e reject separados, é uma struct com os métodos
/// Você pode chamar resolve ou reject de outra thread, mas se criar a promessa em outra thread produz uma promessa já rejeitada
pub enum Promise<T> {
    Resolved(T),
    Rejected(Error),
    Pending(oneshot::Receiver<Result<T, Error>>),
}

impl<T: Send + 'static> Promise<T> {
    pub fn new<C>(run: C) -> Self 
    where 
        C: FnOnce(PromiseResolveReject<T>) + 'static,
    {
        let (tx, rx) = oneshot::channel::<Result<T, Error>>();
        // A promessa inicia o processamento IMEDIATAMENTE em uma nova thread
        // Eu sei que JS não faz assim mas uma coisa por vez né (Futuramente colocar esse processamento em outro lugar?)
        if let Err(e) = EventLoop::spawn(move || {
            run(PromiseResolveReject { tx });
            Ok(())
        }) {
            return Self::Rejected(e);
        }

        Self::Pending(rx)
    }

    pub fn resolve(value: T) -> Self {
        Self::Resolved(value)
    }

    pub fn reject(err: Error) -> Self {
        Self::Rejected(err)
    }

    /// Método auxiliar para encadear operações após a resolução ou rejeição da promessa, sem bloquear a thread atual
    fn after_waiting<F, K>(self, f: F) -> Promise<K>
    where
        F: FnOnce(Result<T, Error>) -> Promise<K> + 'static,
        K: Send + 'static,
    {
        match self {
            Self::Resolved(value) => f(Ok(value)),
            Self::Rejected(err) => f(Err(err)),
            Self::Pending(_) => {
                let (tx, rx) = oneshot::channel::<Result<K, Error>>();
                
                if let Err(e) = self.wait_event_loop(move |result| {
                    let next_promise = f(result);
                    next_promise.wait_event_loop(move |next_result| {
                        match next_result {
                            Ok(v) => tx.send(Ok(v)).ok(),
                            Err(err) => tx.send(Err(err)).ok(),
                        };
                    });
                }) {
                    return Promise::Rejected(e);
                }

                Promise::Pending(rx)
            }
        }
    }

    pub fn then<K: Send + 'static>(
        self,
        f: impl FnOnce(T) -> Promise<K> + 'static
    ) -> Promise<K> {
        self.after_waiting(move |result| {
            match result {
                Ok(value) => f(value),
                Err(err) => Promise::Rejected(err),
            }
        })
    }

    pub fn catch(self, f: impl FnOnce(Error) -> Promise<T> + 'static) -> Promise<T> {
        self.after_waiting(move |result| {
            match result {
                Ok(value) => Promise::Resolved(value),
                Err(err) => f(err),
            }
        })
    }

    pub fn finally(self, f: impl FnOnce() + 'static) -> Self {
        self.after_waiting(move |result| {
            f();
            match result {
                Ok(value) => Promise::Resolved(value),
                Err(err) => Promise::Rejected(err),
            }
        })
    }

    pub fn wait_blocking(self) -> Result<T, Error> {
        match self {
            Self::Resolved(value) => Ok(value),
            Self::Rejected(err) => Err(err),
            Self::Pending(rx) => { 
                match rx.recv() {
                    Ok(value) => value,
                    Err(err) => Err(Box::new(err)),
                }
            }
        }
    }

    pub fn wait_event_loop<F>(self, f: F) -> Result<(), Error>
    where
        F: FnOnce(Result<T, Error>) + 'static,
    {
        match self {
            Self::Resolved(value) => {
                f(Ok(value));
            },
            Self::Rejected(err) => {
                f(Err(err));
            }
            Self::Pending(rx) => { 
                let mut f = Some(f);
                EventLoop::set_interval(move || {
                    match rx.try_recv() {
                        Ok(result) => {
                            f.take().unwrap()(result);
                            Ok(false) // Stop the interval
                        },
                        Err(oneshot::TryRecvError::Empty) => {
                            // Not ready yet, keep waiting
                            Ok(true)
                        },
                        Err(oneshot::TryRecvError::Disconnected) => {
                            // The original promise was dropped without resolving or rejecting
                            f.take().unwrap()(Err(Box::new(io::Error::new(io::ErrorKind::Other, "Original promise was dropped"))));
                            Ok(false) // Stop the interval
                        }
                    }
                }, 1)?;
            }
        }

        Ok(())
    }
}
