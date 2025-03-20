// https://rust-unofficial.github.io/too-many-lists/
use std::fmt;

struct LinkNode {
    value: i32,
    next: Option<Box<LinkNode>>
}

impl fmt::Debug for LinkNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, next: {}]", self.value, if let Some(next) = &self.next { next.value } else { -1 })
    }
}

pub struct LinkedList {
    first: Option<Box<LinkNode>>,
    length: i32
}

impl LinkedList {
    pub fn new() -> LinkedList {
        LinkedList { first: None, length: 0 }
    }

    pub fn from(values: &[i32]) -> LinkedList {
        let mut ret = LinkedList::new();
        ret.push_values(values);
        return ret;
    }

    fn node(value: i32, next: Option<Box<LinkNode>>) -> Box<LinkNode> {
        Box::new(LinkNode { value: value, next: next })
    }

    pub fn _iter(&self) -> LinkedListIterator {
        LinkedListIterator { 
            atual: self.first.as_ref()
        }
    }

    /**
     * Insere um elemento no início da lista
     */
    pub fn push(&mut self, value: i32) {
        self.first = Some(LinkedList::node(
            value, 
            self.first.take()
        ));
        self.length += 1;
    }

    /**
     * Insere vários elementos no início da lista
     */
    pub fn push_values(&mut self, values: &[i32]) {
        for value in values.iter() {
            self.push(*value);
        }
    }

    /**
     * Remove um elemento do início da lista 
     */
    pub fn pop(&mut self) -> i32 {
        match self.first.take() {
            Some(first) => {
                self.first = first.next;
                self.length -= 1;
                return first.value;
            },
            None => {
                panic!("lista vazia!");
            },
        }
    }

    /**
     * Obtêm o valor do início da lista sem de fato tirar da lista
     */
    pub fn peek(&self) -> i32 {
        match &self.first {
            Some(first) => {
                return first.value;
            },
            None => {
                panic!("lista vazia!");
            },
        }
    }

    pub fn len(&self) -> i32 {
        self.length
    }

    pub fn debug(&self) {
        println!("LinkedList [length:{}]",self.len());
        if let Some(first) = &self.first {
            let mut atual: &Box<LinkNode> = first;
            loop {
                println!("\t{:?}",atual);
    
                if let Some(next) = &atual.next {
                    atual = next;
                } else {
                    break;
                }
            }
        } else {
            println!("\tEmpty!");
        }
    }
}
// 'a lifetime, atual deve viver tanto quanto a instância da struct
pub struct LinkedListIterator<'a> {
    atual: Option<&'a Box<LinkNode>>
}

impl<'a> Iterator for LinkedListIterator<'a> {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(atual) = &self.atual {
            let ret = Some(atual.value);

            self.atual = atual.next.as_ref();
            ret
        } else {
            None
        }
    }
}

pub fn _test_linked_list() {
    let mut lista = LinkedList::from(&[1,2,3]);
    lista.push(4);
    lista.push(5);

    println!("Lista:");
    assert_eq!(lista.len(), 5);
    assert_eq!(lista.peek(), 5);
    lista.debug();

    assert_eq!(lista.pop(), 5);
    assert_eq!(lista.pop(), 4);
    assert_eq!(lista.pop(), 3);
    assert_eq!(lista.pop(), 2);
    assert_eq!(lista.pop(), 1);

    println!("Lista depois do pop:");
    assert_eq!(lista.len(), 0);
    lista.debug();
}