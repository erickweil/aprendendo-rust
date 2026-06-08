use std::{pin::Pin, task::{Context, Poll}};

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