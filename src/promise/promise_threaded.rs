use std::sync::{Arc, Mutex};

use crate::promise::{BoxedError};
pub struct PromiseResolveReject<T> {
    tx: oneshot::Sender<Result<T, BoxedError>>
}

impl<T> PromiseResolveReject<T> {
    pub fn resolve(self, value: T) {
        self.tx.send(Ok(value)).ok();
    }

    pub fn reject(self, err: BoxedError) {
        self.tx.send(Err(err)).ok();
    }
}

/// Promessa com comportamento similar ao JavaScript, mas é multithreaded
/// A promessa inicia o processamento IMEDIATAMENTE em uma nova thread
/// Não existe resolve e reject separados, é uma struct com os métodos
/// O método `wait_blocking` bloqueia a thread atual até que a promessa seja resolvida ou rejeitada, retornando o resultado ou o erro
pub enum Promise<T> {
    Resolved(T),
    Rejected(BoxedError),
    Pending(oneshot::Receiver<Result<T, BoxedError>>),
}

impl<T: Send + 'static> Promise<T> {
    pub fn new<C>(run: C) -> Self 
    where 
        C: FnOnce(PromiseResolveReject<T>) + Send + 'static,
    {
        let (tx, rx) = oneshot::channel::<Result<T, BoxedError>>();
        // A promessa inicia o processamento IMEDIATAMENTE de forma síncrona
        run(PromiseResolveReject { tx });        

        Self::Pending(rx)
    }

    pub fn resolve(value: T) -> Self {
        Self::Resolved(value)
    }

    pub fn reject(err: BoxedError) -> Self {
        Self::Rejected(err)
    }

    /// Método auxiliar para encadear operações após a resolução ou rejeição da promessa, sem bloquear a thread atual
    fn after_waiting<F, K>(self, f: F) -> Promise<K>
    where
        F: FnOnce(Result<T, BoxedError>) -> Promise<K> + Send + 'static,
        K: Send + 'static,
    {
        match self {
            Self::Resolved(value) => f(Ok(value)),
            Self::Rejected(err) => f(Err(err)),
            Self::Pending(_) => {
                let (tx, rx) = oneshot::channel::<Result<K, BoxedError>>();

                //return Promise::new(move |e| {
                std::thread::spawn(move || {
                    let result = self.wait_blocking();
                    let next_promise = f(result);
                    match next_promise.wait_blocking() {
                        Ok(v) => tx.send(Ok(v)).ok(),
                        Err(err) => tx.send(Err(err)).ok(),
                    };
                });

                Promise::Pending(rx)
            }
        }
    }

    pub fn then<K: Send + 'static>(
        self,
        f: impl FnOnce(T) -> Promise<K> + Send + 'static
    ) -> Promise<K> {
        self.after_waiting(move |result| {
            match result {
                Ok(value) => f(value),
                Err(err) => Promise::Rejected(err),
            }
        })
    }

    pub fn catch(self, f: impl FnOnce(BoxedError) -> Promise<T> + Send + 'static) -> Promise<T> {
        self.after_waiting(move |result| {
            match result {
                Ok(value) => Promise::Resolved(value),
                Err(err) => f(err),
            }
        })
    }

    pub fn finally(self, f: impl FnOnce() + Send + 'static) -> Self {
        self.after_waiting(move |result| {
            f();
            match result {
                Ok(value) => Promise::Resolved(value),
                Err(err) => Promise::Rejected(err),
            }
        })
    }

    pub fn wait_blocking(self) -> Result<T, BoxedError> {
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
}
