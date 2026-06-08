//! A simple async executor implementation using a single thread and cooperative multitasking.
//! 
//! Based on the video below
//! https://www.youtube.com/watch?v=yfcJGEISsLc
use std::{
    cell::{OnceCell, RefCell},
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex, mpsc::{self}
    },
    task::{Context, Poll, Wake, Waker},
};

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Identificador único de uma tarefa dentro do executor.
pub type TaskId = u64;

struct Task {
    future: BoxedFuture<'static, ()>,
    waker: Waker
}

thread_local! {
    static EXECUTOR_DATA: OnceCell<(RefCell<AsyncExecutor>, mpsc::Receiver<TaskId>)> =
        OnceCell::new();
}

pub struct AsyncExecutor {
    /// Futures vivas, indexadas por TaskId. Ficam nesta thread — podem ser `!Send`.
    tasks: HashMap<TaskId, Task>,
    /// Fila de tarefas prontas para serem poladas.
    ready_queue: VecDeque<TaskId>,
    /// Ponta de envio do canal. Clonada para cada Waker criado.
    sender: mpsc::Sender<TaskId>,
    /// Contador para gerar TaskIds únicos.
    next_id: TaskId,
}

// ---------------------------------------------------------------------------
// Waker: Send + Sync
// ---------------------------------------------------------------------------

/// Dados do Waker: apenas um ID e a ponta de envio do canal.
struct WakerData {
    task_id: TaskId,
    sender: mpsc::Sender<TaskId>,
}

impl Wake for WakerData {
    fn wake(self: Arc<Self>) {
        // Envia o ID pelo canal; o executor acordará e agendará a tarefa.
        let _ = self.sender.send(self.task_id);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.sender.send(self.task_id);
    }
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

impl AsyncExecutor {
    pub fn with_current<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<AsyncExecutor>, &mpsc::Receiver<TaskId>) -> R,
    {
        EXECUTOR_DATA.with(|cell| {
            let (executor, receiver) = cell.get_or_init(|| {
                let (sender, receiver) = mpsc::channel();
                (
                    RefCell::new(AsyncExecutor {
                        tasks: HashMap::new(),
                        ready_queue: VecDeque::new(),
                        sender,
                        next_id: 0,
                    }),
                    receiver,
                )
            });
            f(executor, receiver)
        })
    }

    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        AsyncExecutor::with_current(|executor,_| {
            let mut executor = executor.borrow_mut();
            let id = executor.next_id;
            executor.next_id += 1;

            let waker = Waker::from(Arc::new(WakerData {
                task_id: id,
                sender: executor.sender.clone(),
            }));

            executor.tasks.insert(id, Task {
                future: Box::pin(future),
                waker
            });
            executor.ready_queue.push_back(id);
        });
    }

    /// Roda o executor, processando as tarefas até que todas sejam concluídas.
    pub fn run() -> Result<(), mpsc::RecvError> {
        loop {
            // --- Fase de poll: processa todas as tarefas prontas ---
            loop {
                // Obtém uma task que está pronta
                let Some((task_id, mut task)) = AsyncExecutor::with_current(|executor, _| {
                    let mut executor = executor.borrow_mut();
                    let id = executor.ready_queue.pop_front()?;
                    let task = executor.tasks.remove(&id)?;
                    Some((id, task))
                }) else {
                    break; // Nenhuma tarefa pronta, pode parar
                };

                let mut cx = Context::from_waker(&task.waker);
                match task.future.as_mut().poll(&mut cx) {
                    Poll::Ready(()) => {
                        // Tarefa concluída — descarta a Future.
                    }
                    Poll::Pending => {
                        // Reinsere a Future; o Waker a reagendará pelo canal quando pronta.
                        AsyncExecutor::with_current(|executor, _| {
                            executor.borrow_mut().tasks.insert(task_id, task);
                        });
                    }
                }
            }

            // --- Verifica se há tarefas restantes ---
            if AsyncExecutor::with_current(|executor, _| executor.borrow().tasks.is_empty()) {
                return Ok(());
            }

            // --- Aguarda (bloqueando) até pelo menos um Waker disparar ---
            AsyncExecutor::with_current(|executor, receiver| {
                let mut executor = executor.borrow_mut();

                let first = receiver.recv()?;
                executor.ready_queue.push_back(first);
                // Drena o restante do buffer para minimizar idas e vindas ao canal.
                for id in receiver.try_iter() {
                    executor.ready_queue.push_back(id);
                }

                Ok(())
            })?;
        }
    }
}