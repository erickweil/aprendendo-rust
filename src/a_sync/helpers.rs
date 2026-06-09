use std::{future::Future, ops::DerefMut, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::{Duration, Instant}};

pub struct Yield {
    ticks: u8,
}

impl Future for Yield {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ticks > 0 {
            // Decrementa o contador de ticks. Na primeira vez que for polled, ele vai ser 1, então vai entrar aqui e decrementar para 0.
            self.ticks -= 1;
            
            // ACORDAR
            cx.waker().wake_by_ref();
            
            Poll::Pending
        } else {
            // Na próxima rodada do EventLoop, a execução cai aqui e avança
            Poll::Ready(())
        }
    }
}

impl Yield {
    /// Future que 'pausa' por 1 tick do event loop
    /// Ex: `Yield::now().await` cede a execução para outras tarefas e só volta a executar na próxima rodada do EventLoop.
    pub fn now() -> impl Future<Output = ()> {
        Self { ticks: 1 }
    }

    pub fn ticks(ticks: u8) -> impl Future<Output = ()> {
        Self { ticks }
    }
}


pub struct Sleep {
    deadline: Instant,
    timer_started: bool,
}

impl Sleep {
    /// Future que 'pausa' por um tempo determinado
    /// Ex: `Sleep::duration(Duration::from_secs(1)).await` cede a execução por 1 segundo e só volta a executar depois disso.
    /// Obs: É uma implementação de exemplo, o timer é implementado com uma thread separada. 
    pub fn duration(dur: Duration) -> Self {
        Self {
            deadline: Instant::now() + dur,
            timer_started: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Já passou do deadline? Pronto.
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }

        // Spawna a thread de timer apenas uma vez
        if !self.timer_started {
            self.timer_started = true;

            let deadline = self.deadline;
            let waker = cx.waker().clone();
            thread::spawn(move || {
                let now = Instant::now();
                if deadline > now {
                    thread::sleep(deadline - now);
                }
                // Acorda o executor
                waker.wake();
            });
        }

        Poll::Pending
    }
}