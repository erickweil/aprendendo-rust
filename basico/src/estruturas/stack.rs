// https://rust-unofficial.github.io/too-many-lists/
use std::fmt;

pub trait Stack<T> {
    /**
     * Insere um elemento no topo da pilha
     */
    fn push(&mut self, value: T);

    /**
     * Insere vários elementos no topo da pilha
     * Obs: para [A,B,C] irá fazer push(A), push(B), push(C)
     */
    fn push_values<const N: usize>(&mut self, values: [T; N]) {
        for value in values.into_iter() {
            self.push(value);
        }
    }

    /**
     * Remove um elemento do topo da pilha 
     */
    fn pop(&mut self) -> Option<T>;

    /**
     * 'Empresta' o valor do topo da pilha. não irá remover
     */
    fn peek(&self) -> Option<&T>;

    /**
     * Obtêm o tamanho da pilha
     */
    fn len(&self) -> i32;
}

struct LinkNode<T> {
    value: T,
    next: Option<Box<LinkNode<T>>>
}

impl<T: fmt::Debug> fmt::Debug for LinkNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} next: ", self.value)?;
        if let Some(next) = &self.next {
            write!(f, "{:?}", next.value)?;
        } else {
            write!(f, "null")?;
        }
        Ok(())
    }
}

pub struct LinkedStack<T> {
    first: Option<Box<LinkNode<T>>>,
    length: i32
}

impl<T> LinkedStack<T> {
    pub fn new() -> LinkedStack<T> {
        LinkedStack { first: None, length: 0 }
    }

    pub fn from<const N: usize>(values: [T; N]) -> LinkedStack<T> {
        let mut ret = LinkedStack::new();
        ret.push_values(values);
        return ret;
    }

    fn node(value: T, next: Option<Box<LinkNode<T>>>) -> Box<LinkNode<T>> {
        Box::new(LinkNode { value: value, next: next })
    }

    pub fn iter(&self) -> LinkedListIterator<T> {
        LinkedListIterator { 
            atual: self.first.as_ref()
        }
    }
}

impl<T> Stack<T> for LinkedStack<T> {
    fn push(&mut self, value: T) {
        // Necessário usar Option.take() para obter o valor ao mesmo tempo que transforma ele em None
        // Isso é para o valor em nenhum instante ter dois donos, e evitar fazer uma nova cópia
        self.first = Some(LinkedStack::node(
            value, 
            self.first.take()
        ));
        self.length += 1;
    }

    fn pop(&mut self) -> Option<T> {
        match self.first.take() {
            Some(first) => {
                self.first = first.next;
                self.length -= 1;
                return Some(first.value);
            },
            None => { return None },
        }
    }

    fn peek(&self) -> Option<&T> {
        match &self.first {
            Some(first) => { return Some(&first.value); },
            None => { return None },
        }
    }

    fn len(&self) -> i32 {
        self.length
    }
}

impl<T: fmt::Debug> fmt::Debug for LinkedStack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinkedStack [length:{}]", self.len())?;
        if let Some(first) = &self.first {
            let mut atual: &Box<LinkNode<T>> = first;
            loop {
                writeln!(f)?;
                write!(f, "\t{:?}",atual)?;
    
                if let Some(next) = &atual.next {
                    atual = next;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}

// 'a lifetime, atual deve viver tanto quanto a instância da struct
pub struct LinkedListIterator<'a,T> {
    atual: Option<&'a Box<LinkNode<T>>>
}

impl<'a,T> Iterator for LinkedListIterator<'a,T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(atual) = self.atual {
            let ret: Option<&T> = Some(&atual.value);

            self.atual = atual.next.as_ref();
            ret
        } else {
            None
        }
    }
}

// cargo test --package basico --bin basico -- estruturas::linked_stack::test --show-output
#[cfg(test)]
mod test {
    use super::LinkedStack;
    use super::Stack;

    pub fn run_stack_tests<T: Stack<char>>(stack: &mut T) {
        assert_eq!(stack.peek(), None);

        stack.push_values(['A','B','C']);
        stack.push('D');
        stack.push('E');

        assert_eq!(stack.len(), 5);
        assert_eq!(stack.peek(), Some(&'E'));

        assert_eq!(stack.pop(), Some('E'));
        assert_eq!(stack.pop(), Some('D'));
        stack.push('F');
        assert_eq!(stack.pop(), Some('F'));
        assert_eq!(stack.pop(), Some('C'));
        assert_eq!(stack.pop(), Some('B'));
        assert_eq!(stack.pop(), Some('A'));
        assert_eq!(stack.pop(), None);

        assert_eq!(stack.len(), 0);
    }

    #[test]
    pub fn linked_stack() {
        let mut stack = LinkedStack::from([]);
        run_stack_tests(&mut stack);
    }
}