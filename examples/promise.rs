use std::{any::Any, sync::{Arc, Mutex}};

type Error = Box<dyn std::error::Error>;
type Callback<T> = Box<dyn FnOnce(T) + Send + 'static>;

pub struct Resolve<T>(Option<Callback<T>>);
pub struct Reject(Option<Callback<Error>>);

type Runner<T>   = Box<dyn FnOnce(Resolve<T>, Reject) + Send + 'static>;

impl<T> Resolve<T> {
    pub fn call(mut self, value: T) {
        if let Some(f) = self.0.take() { f(value); }
    }
}

impl Reject {
    pub fn call(mut self, err: Error) {
        if let Some(f) = self.0.take() { 
            f(err); 
        } else {
            panic!("Promise rejected with error: {}", err);
        }
    }
}

// Trait para converter qualquer valor em uma Promise
pub trait IntoPromise<T: 'static> {
    fn into_promise(self) -> Promise<T>;
}

// Qualquer valor vira uma Promise já resolvida
impl<T: Send + 'static> IntoPromise<T> for T {
    fn into_promise(self) -> Promise<T> {
        let value = self;
        Promise::new(move |resolve, _| resolve.call(value))
    }
}

// Promise<T> já é uma Promise — só passa adiante
impl<T: Send + 'static> IntoPromise<T> for Promise<T> {
    fn into_promise(self) -> Promise<T> {
        self
    }
}

pub struct Promise<T: 'static> {
    _run:   Option<Runner<T>>,
    _then:  Option<Callback<T>>,
}

impl<T: 'static> Promise<T> {
    pub fn new(run: impl FnOnce(Resolve<T>, Reject) + Send + 'static) -> Self {
        Self {
            _run:   Some(Box::new(run)),
            _then:  None,
        }
    }

    pub fn then<K: 'static, R: IntoPromise<K>>(
        mut self,
        f: impl FnOnce(T) -> R + Send + 'static
    ) -> Promise<K> {
        Promise::new(move |resolve, reject| {
            assert!(self._then.is_none(), "then() já foi definido");
            self._then = Some(Box::new(move |value| {
                let mut result = f(value).into_promise();
                result._then = Some(Box::new(move |k| resolve.call(k)));
                result.run();
            }));
            self.run();
        })
    }

    pub fn run(mut self) {
        let runner = self._run.take().expect("run() já foi chamado");

        let resolve = Resolve(self._then.take());
        let reject = Reject(None);

        // Nenhuma referência a self sobrevive após essa linha
        runner(resolve, reject);
    }

    pub fn wait_blocking<K: Send>(promise: Promise<K>) -> K {
        let (tx, rx) = std::sync::mpsc::channel();

        promise.then(move |value| {
            tx.send(value).unwrap();
        }).run();

        rx.recv().unwrap()
    }
}

fn esperar(ms: u64) -> Promise<()> {
    Promise::new(move |resolve, reject| {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            resolve.call(());
        });
    })
}

fn executar() -> Promise<()> {
    println!("Contando...");
    esperar(1000).then(move |_: ()| {
        println!("1...");
        esperar(1000)
    }).then(move |_: ()| {
        println!("2...");
        esperar(1000)
    }).then(move |_: ()| {
        println!("3...");
        esperar(1000)
    })
}

fn main() {
    Promise::<()>::wait_blocking(executar());
    println!("Tudo terminou!");
}
