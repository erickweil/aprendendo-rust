use std::marker::PhantomData;

pub trait Queue<T> {
    /**
     * Insere um elemento no final da fila
     */
    fn enqueue(&mut self, value: T);

    /**
     * Insere vários elementos no final da fila
     * Obs: para [A,B,C] irá fazer add(A), add(B), add(C)
     */
    fn enqueue_values<const N: usize>(&mut self, values: [T; N]) {
        for value in values.into_iter() {
            self.enqueue(value);
        }
    }

    /**
     * Remove um elemento do início da fila
     */
    fn dequeue(&mut self) -> Option<T>;

    /**
     * 'Empresta' o valor do início da fila (Do lado que remove, o próximo que seria retornado por dequeue())
     */
    fn head(&self) -> Option<&T>;

    /**
     * 'Empresta' o valor do final da fila (Do lado que insere, o último que foi inserido com enqueue())
     */
    fn tail(&self) -> Option<&T>;
}

pub struct DoubleStackQueue<T> {
    entrada: Vec<T>,
    saida: Vec<T>
}

impl<T> DoubleStackQueue<T> {
    pub fn new() -> DoubleStackQueue<T> {
        DoubleStackQueue {
            entrada: Vec::new(),
            saida: Vec::new()
        }
    }

    pub fn from<const N: usize>(values: [T; N]) -> DoubleStackQueue<T> {
        let mut ret = DoubleStackQueue::new();
        ret.enqueue_values(values);
        return ret;
    }

    pub fn iter(&self) -> DoubleStackQueueIterator<T> {
        DoubleStackQueueIterator { queue: &self, index: 0 }
    }

    fn len(&self) -> i32 {
        return (self.entrada.len() + self.saida.len()) as i32;
    }
}

impl<T> Queue<T> for DoubleStackQueue<T> {
    fn enqueue(&mut self, value: T) {
        self.entrada.push(value);
    }

    fn tail(&self) -> Option<&T> {
        if self.entrada.len() > 0 {
            return self.entrada.last();
        }
        return self.saida.first();        
    }

    fn head(&self) -> Option<&T> {
        if self.saida.len() <= 0 {
            return self.entrada.first();
        }
        return self.saida.last();
    }

    fn dequeue(&mut self) -> Option<T> {
        if self.saida.len() <= 0 { 
            if self.entrada.len() <= 0 { return None; }

            // Despejar entrada na saída
            while self.entrada.len() > 0 {
                self.saida.push(self.entrada.pop().unwrap());
            }
        }
        return self.saida.pop();
    }
}

pub struct DoubleStackQueueIterator<'a,T> {
    queue: &'a DoubleStackQueue<T>,
    index: usize
}

impl<'a,T> Iterator for DoubleStackQueueIterator<'a,T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let ret: Option<Self::Item>;
        let saida_len = self.queue.saida.len();
        let entrada_len = self.queue.entrada.len();
        if self.index < saida_len {
            // atravessar do fim ao início
            let saida_index = saida_len - self.index - 1;
            ret = Some(&self.queue.saida[saida_index]);
        } else if self.index < saida_len + entrada_len {
            // atravessar do início ao fim
            let entrada_index = self.index - saida_len;
            ret = Some(&self.queue.entrada[entrada_index]);
        } else {
            ret = None;
        }

        self.index += 1;
        ret
    }
}

pub fn run_queue_tests<T: Queue<char>>(queue: &mut T) {
    for iter in 0..3 {
        assert_eq!(queue.head(), None);
        assert_eq!(queue.tail(), None);

        queue.enqueue_values(['A','B','C']);
        queue.enqueue('D');
        queue.enqueue('E');

        //assert_eq!(queue.len(), 5);
        assert_eq!(queue.head(), Some(&'A'));
        assert_eq!(queue.tail(), Some(&'E'));

        assert_eq!(queue.dequeue(), Some('A'));
        assert_eq!(queue.dequeue(), Some('B'));
        queue.enqueue('F');
        assert_eq!(queue.dequeue(), Some('C'));
        assert_eq!(queue.dequeue(), Some('D'));
        assert_eq!(queue.dequeue(), Some('E'));
        assert_eq!(queue.dequeue(), Some('F'));
        assert_eq!(queue.dequeue(), None);
    }
    //assert_eq!(queue.len(), 0);
}

#[cfg(test)]
mod test {
    use super::run_queue_tests;
    use super::DoubleStackQueue;
    use super::Queue;

    #[test]
    pub fn double_stack_queue() {
        let mut queue = DoubleStackQueue::from([]);
        run_queue_tests(&mut queue);
    }

    #[test]
    pub fn double_stack_queue_iter() {
        let arr = ['A','B','C','D','E'];
        let mut queue = DoubleStackQueue::from(arr);
        
        let mut counter = 0;
        for value in queue.iter() {
            assert_eq!(value, &arr[counter]);
            counter += 1;
        }
    }
}