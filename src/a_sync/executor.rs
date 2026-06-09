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

use crate::estruturas::VecPool;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Identificador único de uma tarefa dentro do executor.
pub type TaskId = usize;

struct Task {
    future: BoxedFuture<'static, ()>,
    waker: Waker
}

thread_local! {
    static EXECUTOR_DATA: OnceCell<RefCell<AsyncExecutor>> = const { OnceCell::new() };
}

pub struct AsyncExecutor {
    /// Futures vivas, indexadas por TaskId. Ficam nesta thread — podem ser `!Send`.
    /// VecPool é minha própria implementação de Slab/slotmap
    tasks: VecPool<Option<Task>>,
    /// Ponta de envio do canal. Clonada para cada Waker criado.
    sender: mpsc::Sender<TaskId>,
    /// Ponta de recepção do canal, usada para receber notificações de Wakers.
    receiver: mpsc::Receiver<TaskId>
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
        F: FnOnce(&RefCell<AsyncExecutor>) -> R,
    {
        EXECUTOR_DATA.with(|cell| {
            let executor = cell.get_or_init(|| {
                let (sender, receiver) = mpsc::channel();
                RefCell::new(AsyncExecutor {
                    tasks: VecPool::new(),
                    sender,
                    receiver
                })
            });
            f(executor)
        })
    }

    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        AsyncExecutor::with_current(|executor| {
            let mut executor = executor.borrow_mut();
            let task_id = executor.tasks.alloc_node(None);

            let waker = Waker::from(Arc::new(WakerData {
                task_id: task_id,
                sender: executor.sender.clone(),
            }));

            let node = executor.tasks.get_mut_node(task_id).unwrap();
            *node = Some(Task {
                future: Box::pin(future),
                waker
            });

            executor.sender.send(task_id).ok();
        });
    }

    /// Roda o executor, processando as tarefas até que todas sejam concluídas.
    /// Só deve chamar uma vez, e bloqueia a thread até que todas as tarefas terminem.
    pub fn run() -> Result<(), mpsc::RecvError> {
        loop {
            // --- Aguarda (bloqueando) até pelo menos um Waker disparar ---
            let Some((task_id, mut task)) = AsyncExecutor::with_current(|executor| {
                let mut executor = executor.borrow_mut();

                if executor.tasks.len() == 0 {
                    return Ok(None);
                }

                loop {
                    let task_id = executor.receiver.recv()?;
                    let maybe_task_slot = executor.tasks.get_mut_node(task_id);
                    if let Some(task_slot) = maybe_task_slot {
                        let task = task_slot.take().unwrap();
                        return Ok(Some((task_id, task)));
                    } else {
                        eprintln!("Warning: Received task_id {} but it was not found in tasks map, ignoring it", task_id);
                    }
                }
            })? else {
                // Não há tarefas restantes, o executor pode encerrar.
                return Ok(());
            };
            
            let mut cx = Context::from_waker(&task.waker);
            if let Poll::Pending = task.future.as_mut().poll(&mut cx) {
                // Reinsere a Future no mesmo slot, pois o Waker a reagendará pelo canal usando o mesmo ID quando pronta.
                AsyncExecutor::with_current(|executor| {
                    let mut executor = executor.borrow_mut();
                    let task_slot = executor.tasks.get_mut_node(task_id).unwrap();
                    *task_slot = Some(task);
                });
            } else {
                // Tarefa concluída, Libera o slot do pool.
                AsyncExecutor::with_current(|executor| {
                    let mut executor = executor.borrow_mut();
                    executor.tasks.free_node(task_id);
                });
            }
        }
    }
}