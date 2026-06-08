use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use crate::promise::{EventLoop, EventLoopError, TaskError};

pub struct EventLoopAsyncTask {
    // A future precisa ser Pin e Boxed para não mudar de endereço de memória
    // O Mutex é exigido pois a trait Wake exige que o tipo seja Send + Sync
    future: Mutex<Pin<Box<dyn Future<Output = Result<(), TaskError>> + Send + 'static>>>,
}

impl EventLoopAsyncTask {
    pub fn new<F: IntoFuture>(future: F) -> Arc<Self> 
    where
        F::IntoFuture: Future<Output = Result<(), TaskError>> + Send + 'static,
    {
        Arc::new(Self {
            future: Mutex::new(Box::pin(future.into_future())),
        })
    }

    /// Obs: Chamar poll_task após retornar Some(T) irá dar panic!
    pub fn poll_task(task: Arc<EventLoopAsyncTask>) -> Option<Result<(), TaskError>> {
        // Converter Arc<AsyncTask> para Waker
        let waker: Waker = task.clone().into();
        let mut cx = Context::from_waker(&waker);

        // Mutex Lock
        let mut future = task.future.lock().unwrap();

        match future.as_mut().poll(&mut cx) {
            Poll::Pending => {
                // Ainda não terminou
                return None;
            }
            Poll::Ready(result) => {
                // A tarefa terminou, retornamos o valor
                return Some(result);
            }
        }
    }
}

impl Wake for EventLoopAsyncTask {
    fn wake(self: Arc<Self>) {        
        // Quando a tarefa é acordada, enfileira um poll para ela mesma no EventLoop
        let _ = EventLoop::try_spawn_remote(move || {
            if let Some(result) = EventLoopAsyncTask::poll_task(self) {
                result
            } else {
                Ok(())
            }
        });
    }
}
