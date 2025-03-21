use super::{Queue, Stack, VecPool, NULL_INDEX};

/**
 * Lista duplamente encadeada sem o uso de ponteiros (isso mesmo nada de Box ou Rc)
 * 
 * Funcionamento:
 * - IndexNode contém um Option<T>, índice do anterior e índice do próximo. (Ou NULL)
 * - Um Vec<IndexNode> armazena todos os nós (incluindo os 'liberados')
 * 
 * Para não crescer o vetor sem limites com valores 'liberados' em adições/remoções frequentes, 
 * utiliza-se de uma lógica de 'alocação' de nós:
 * - Ao criar um novo nó, se houver um vazio, utiliza ele, caso contrário faz push no Vec
 * - Ao remover um nó, registra que este nó está vazio
 * 
 * Para não ser lento procurar os nós vazios, aproveita a própria estrutura como ao mesmo tempo uma pilha (Linked) de valores vazios.
 * - Armazena o índice do último valor vazio (topo da pilha), ou NULL se não tem
 * - Cada valor vazio contém o next do próximo valor vazio, ou NULL
 * - Ao criar novo nó é realizado 'push' nesta pilha ligada, e ao remover é feito 'pop'
 * 
 * Referência da ideia:
 * https://stackoverflow.com/questions/3002764/is-a-linked-list-implementation-without-using-pointers-possible-or-not
 * Q: Is a Linked-List implementation without using pointers possible or not?
 * 
 * https://stackoverflow.com/a/13158115
 * A: Yes you can, it is not necessary to use pointers for a link list. It is possible to link a list without using pointers. You can statically allocate an array for the nodes, and instead of using next and previous pointer, you can just use indexes. You can do that to save some memory, if your link list is not greater than 255 for example, you can use 'unsigned char' as index (referencing C), and save 6 bytes for next and previous indications.
 * You may need this kind of array in embedded programming, since memory limitations can be troublesome sometimes.
 * Also keep in mind that your link list nodes will not necessary be contiguous in the memory.
 * Let's say your link list will have 60000 nodes. Allocating a free node from the array using a linear search should be inefficient. Instead, you can just keep the next free node index everytime:
 * Just initialize your array as each next index shows the current array index + 1, and firstEmptyIndex = 0.
 * When allocating a free node from the array, grab the firstEmptyIndex node, update the firstEmptyIndex as next index of the current array index (do not forget to update the next index as Null or empty or whatever after this).
 * When deallocating, update the next index of the deallocating node as firstEmptyIndex, then do firstEmptyIndex = deallocating node index.
 * In this way you create yourself a shortcut for allocating free nodes from the array.
 */
pub struct LinkedList {
    arr: VecPool<LinkedNode>,
    first: i32,
    last: i32
}

pub struct LinkedNode {
    value: char,
    next: i32,
    prev: i32
}

impl LinkedList {
    pub fn new() -> LinkedList {
        LinkedList { 
            arr: VecPool::new(),
            first: NULL_INDEX,
            last: NULL_INDEX
        }
    }

    pub fn clear(&mut self) {
        self.arr.clear();
        self.first = NULL_INDEX;
        self.last = NULL_INDEX;
    }

    fn len(&self) -> i32 {
        self.arr.len()
    }

    pub fn add_first(&mut self, value: char) {
        // Cria o novo nó no array e obtêm o índice dele
        let len = self.arr.len();
        let new_node = self.arr.alloc_node(LinkedNode { value: value, next: NULL_INDEX, prev: NULL_INDEX });
        if len == 0 {
            // Agora início e fim da lista é ele
            self.first = new_node;
            self.last = new_node; 
        } else {
            // 1 - .next do novo nó deve apontar para o antigo início da lista
            self.arr.get_mut_node(new_node).unwrap().next = self.first;
            // 2 - .prev do antigo início deve apontar para o novo nó
            self.arr.get_mut_node(self.first).unwrap().prev = new_node;
            // 3 - Agora o início da lista é o novo nó
            self.first = new_node;
        }
    }

    pub fn add_last(&mut self, value: char) {
        // Cria o novo nó no array e obtêm o índice dele
        let len = self.arr.len();
        let new_node = self.arr.alloc_node(LinkedNode { value: value, next: NULL_INDEX, prev: NULL_INDEX });
        if len == 0 {
            // Agora início e fim da lista é ele
            self.first = new_node;
            self.last = new_node; 
        } else {
            // 1 - .prev do novo nó deve apontar para o antigo final da lista
            self.arr.get_mut_node(new_node).unwrap().prev = self.last;
            // 2 - .next do antigo final deve apontar para o novo nó
            self.arr.get_mut_node(self.last).unwrap().next = new_node;
            // 3 - Agora o final da lista é o novo nó
            self.last = new_node;
        }
    }

    pub fn remove_first(&mut self) -> Option<char> {
        if self.first == NULL_INDEX { return None; }
    
        // MOVE o valor do primeiro nó, deixando None no lugar
        let len = self.arr.len();
        let first_node = self.arr.free_node(self.first);
        if len == 1 {
            // Se era o último nó, limpa a lista
            self.clear();
        } else {
            // 1 - .prev do próximo nó deve apontar para NULL agora
            let next_index = first_node.next;
            self.arr.get_mut_node(next_index).unwrap().prev = NULL_INDEX;
            // 2 - first é o próximo do primeiro agora
            self.first = next_index;
        }

        return Some(first_node.value);
    }

    pub fn remove_last(&mut self) -> Option<char> {
        if self.last == NULL_INDEX { return None; }
    
        // MOVE o valor do último nó, deixando None no lugar
        let len = self.arr.len();
        let last_node = self.arr.free_node(self.last);
        if len == 1 {
            // Se era o último nó, limpa a lista
            self.clear();
        } else {
            // 1 - .next do nó anterior ao último deve apontar para NULL agora
            let prev_index = last_node.prev;
            self.arr.get_mut_node(prev_index).unwrap().next = NULL_INDEX;
            // 2 - last é o anterior ao último agora
            self.last = prev_index;
        }

        return Some(last_node.value);
    }

    pub fn peek_first(&self) -> Option<&char> {
        if self.first == NULL_INDEX { return None; }

        return Some(&self.arr.get_node(self.first).unwrap().value);
    }

    pub fn peek_last(&self) -> Option<&char> {
        if self.last == NULL_INDEX { return None; }

        return Some(&self.arr.get_node(self.last).unwrap().value);
    }
}

impl Stack<char> for LinkedList {
    fn push(&mut self, value: char) { self.add_first(value); }
    fn pop(&mut self) -> Option<char> { return self.remove_first(); }
    fn peek(&self) -> Option<&char> { return self.peek_first(); }
}

impl Queue<char> for LinkedList {
    fn enqueue(&mut self, value: char) { self.add_last(value); }
    fn dequeue(&mut self) -> Option<char> { return self.remove_first(); }
    fn head(&self) -> Option<&char> { return self.peek_first(); }
    fn tail(&self) -> Option<&char> { return self.peek_last(); }
}


#[cfg(test)]
mod test {
    use crate::estruturas::{linked_list::*, run_queue_tests, run_stack_tests};


    #[test]
    pub fn arr_alloc_logic() {
        let mut list = LinkedList::new();
        list.add_last('0');
        for i in 0..10 {
            list.add_last('A');
            list.add_last('B');
            list.add_last('C');
            list.add_first('A');
            list.add_first('B');
            list.add_first('C');
            assert_eq!(list.len(), 7);
            
            assert_eq!(list.remove_last(), Some('C'));
            assert_eq!(list.remove_last(), Some('B'));
            assert_eq!(list.remove_last(), Some('A'));
            assert_eq!(list.remove_first(), Some('C'));
            assert_eq!(list.remove_first(), Some('B'));
            assert_eq!(list.remove_first(), Some('A'));            

            assert_eq!(list.len(), 1);
        }
        
        assert_eq!(list.len(), 1);
        assert_eq!(list.peek_first(), Some(&'0'));

        // Como houve momentos que a lista teve até 7 elementos, esse deve ser o tamanho do array
        //assert_eq!(list.arr.len(), 7);
        //assert_eq!(list.arr.last_empty, 1);

        // Inserir um novo elemento não deve aumentar o array e sim aproveitar os espaços
        list.add_last('D');
        assert_eq!(list.len(), 2);
        //assert_eq!(list.arr.last_empty, 2);
    }

    #[test]
    pub fn linked_list_queue() {
        let mut queue = LinkedList::new();
        run_queue_tests(&mut queue);

        //assert_eq!(queue.last_empty, NULL_INDEX);
        assert_eq!(queue.arr.len(), 0);
    }

    #[test]
    pub fn double_linked_stack() {
        let mut stack = LinkedList::new();
        run_stack_tests(&mut stack);

        //assert_eq!(stack.last_empty, NULL_INDEX);
        assert_eq!(stack.arr.len(), 0);
    }
}