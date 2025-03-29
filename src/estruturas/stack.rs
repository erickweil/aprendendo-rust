// https://rust-unofficial.github.io/too-many-lists/
use std::{fmt, rc::Rc};

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
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
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

/**
 * Single Linked List
 * A ideia é ser uma estrutura estilo pilha, e leve de se criar cópias (uma cópia é só uma referência ao último elemento)
 */
#[derive(Clone)]
pub struct LinkedStack<T> {
    head: Link<T>,
    length: i32
}

impl<T> LinkedStack<T> {
    pub fn new() -> LinkedStack<T> {
        LinkedStack { head: None, length: 0 }
    }

    fn node(value: T, next: Link<T>) -> Rc<Node<T>> {
        Rc::new(Node { value: value, next: next })
    }

    pub fn iter(&self) -> LinkedStackIter<T> {
        LinkedStackIter { 
            atual: self.head.as_ref()
        }
    }

    pub fn len(&self) -> i32 {
        self.length
    }
}

impl<T> Stack<T> for LinkedStack<T> 
where 
    T: Clone
{
    fn push(&mut self, value: T) {
        // Necessário usar Option.take() para obter o valor ao mesmo tempo que transforma ele em None
        // Isso é para o valor em nenhum instante ter dois donos, e evitar fazer uma nova cópia
        self.head = Some(LinkedStack::node(
            value, 
            self.head.take()
        ));
        self.length += 1;
    }

    fn pop(&mut self) -> Option<T> {
        match self.head.take() {
            Some(first) => {
                self.head = first.next.clone();
                self.length -= 1;
                return Some(first.value.clone());
            },
            None => { return None },
        }
    }

    fn peek(&self) -> Option<&T> {
        match self.head.as_ref() {
            Some(first) => { return Some(&first.value); },
            None => { return None },
        }
    }
}

/**
 * You don't actually need to implement Drop if you contain types that implement Drop, and all you'd want to do is call their destructors.
 * All that's handled for us automatically... with one hitch.
 * The automatic handling is going to be bad.
 * Let's consider a simple list: 
 * list -> A -> B -> C
 * When list gets dropped, it will try to drop A, which will try to drop B, which will try to drop C. 
 * Some of you might rightly be getting nervous. This is recursive code, and recursive code can blow the stack!
 */
impl<T> Drop for LinkedStack<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for LinkedStack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinkedStack [length:{}]", self.len())?;
        if let Some(first) = &self.head {
            let mut atual: &Rc<Node<T>> = first;
            loop {
                writeln!(f)?;
                write!(f, "\t refs: {}, {:?}",Rc::strong_count(atual),atual)?;
    
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
pub struct LinkedStackIter<'a,T> {
    atual: Option<&'a Rc<Node<T>>>
}

impl<'a,T> Iterator for LinkedStackIter<'a,T> {
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

pub fn run_stack_tests<T: Stack<char>>(stack: &mut T) {
    for iter in 0..3 {
        assert_eq!(stack.peek(), None);

        stack.push_values(['A','B','C']);
        stack.push('D');
        stack.push('E');

        //assert_eq!(stack.len(), 5);
        assert_eq!(stack.peek(), Some(&'E'));

        assert_eq!(stack.pop(), Some('E'));
        assert_eq!(stack.pop(), Some('D'));
        stack.push('F');
        assert_eq!(stack.pop(), Some('F'));
        assert_eq!(stack.pop(), Some('C'));
        assert_eq!(stack.pop(), Some('B'));
        assert_eq!(stack.pop(), Some('A'));
        assert_eq!(stack.pop(), None);
    }

    //assert_eq!(stack.len(), 0);
}

// cargo test --package basico --bin basico -- estruturas::linked_stack::test --show-output
#[cfg(test)]
mod test {
    use super::run_stack_tests;
    use super::LinkedStack;
    use super::Stack;

    #[test]
    pub fn linked_stack() {
        let mut stack = LinkedStack::new();
        run_stack_tests(&mut stack);
    }

    #[test]
    pub fn copy() {
        let mut stack = LinkedStack::new();
        stack.push_values(['A','B','C']);

        let mut stack_copy = stack.clone();

        stack_copy.push('D');
        stack_copy.push('E');

        assert_eq!(stack.len(), 3);
        assert_eq!(stack_copy.len(), 5);

        //println!("{:?}", stack);
        //println!("{:?}", stack_copy);

        assert_eq!(stack.pop(), Some('C'));
        assert_eq!(stack.pop(), Some('B'));
        assert_eq!(stack.pop(), Some('A'));

        assert_eq!(stack.len(), 0);
        assert_eq!(stack_copy.len(), 5);

        assert_eq!(stack_copy.pop(), Some('E'));
        assert_eq!(stack_copy.pop(), Some('D'));
        assert_eq!(stack_copy.pop(), Some('C'));
        assert_eq!(stack_copy.pop(), Some('B'));
        assert_eq!(stack_copy.pop(), Some('A'));
    }

    #[test]
    pub fn destructors() {
        // The idea is to force a stack overflow when destructors run recursively with a too large list
        // If the Drop impl above didn't work this test would fail
        
        let mut stack = LinkedStack::new();
        let size = 1_000_000;
        for i in 0..size {
            stack.push(i);
        }

        assert_eq!(stack.len(), size);

        // cause the stack be de-allocated
        stack = LinkedStack::new();

        assert_eq!(stack.len(), 0);
    }
}