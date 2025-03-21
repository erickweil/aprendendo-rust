use std::mem::replace;

use rand::distr::slice::Empty;

// valor de índice que será utilizado para representar a ausência de conexão
pub const NULL_INDEX: i32 = -1;

#[derive(Debug)]
enum IndexNode<T> {
    Filled(T),
    Empty(i32)
}

impl<T> IndexNode<T> {
    /*fn move_then_free(&mut self, next_empty: i32) -> Option<T> {
        let moved = self.value.take();
        self.next = NULL_INDEX;

        return moved;
    }*/

    fn try_get(&self) -> Option<&T> {
        match self {
            IndexNode::Filled(value) => Some(value),
            IndexNode::Empty(_) => None,
        }
    }

    fn try_get_mut(&mut self) -> Option<&mut T> {
        match self {
            IndexNode::Filled(value) => Some(value),
            IndexNode::Empty(_) => None,
        }
    }
}

/**
 * Object (Você quis dizer struct?) pooling
 * 
 * Para não ser lento procurar os nós vazios, aproveita a própria estrutura como ao mesmo tempo uma pilha (Linked) de valores vazios.
 * - Armazena o índice do último valor vazio (topo da pilha), ou NULL se não tem
 * - Cada valor vazio contém o next do próximo valor vazio, ou NULL
 * - Ao criar novo nó é realizado 'push' nesta pilha ligada, e ao remover é feito 'pop'
 */
pub struct VecPool<T> {
    arr: Vec<IndexNode<T>>,
    length: i32,
    last_empty: i32
}

impl<T> VecPool<T> {
    pub fn new() -> VecPool<T> {
        VecPool { 
            arr: Vec::new(),
            length: 0,
            last_empty: NULL_INDEX
        }
    }

    /**
     * Aloca um novo nó na lista, aproveitando espaços vazios se possível
     * 
     * Retorna o índice que foi criado
     */
    pub fn alloc_node(&mut self, value: T) -> i32 {
        let node = IndexNode::Filled(value);
        if self.last_empty == NULL_INDEX { // não tem nenhum vazio, adicionar mais um no final do array
            self.arr.push(node);
            self.length += 1;

            return (self.arr.len()-1) as i32;
        } else {
            // fazer pop() da pilha de valores vazios
            // 1 - obter índice do espaço vazio
            let free_node = self.last_empty;

            // 2 - topo da pilha deve apontar para o próximo espaço vazio (ou NULL)
            if let IndexNode::Empty(next) = self.arr[self.last_empty as usize] {
                self.last_empty = next;
            } else {
                panic!("Deveria ser um Empty");
            }

            // Re-Inicializar os valores
            self.arr[free_node as usize] = node;
            self.length += 1;

            return free_node;
        }
    }

    /**
     * Libera um nó da lista tornando um espaço vazio disponível, e retorna o valor que estava nele,
     */
    pub fn free_node(&mut self, node: i32) -> T {
        // fazer push() na pilha de valores vazios
        let ret = replace(&mut self.arr[node as usize], IndexNode::Empty(self.last_empty));
        self.last_empty = node;
        self.length -= 1;

        return match ret {
            IndexNode::Filled(value) => value,
            IndexNode::Empty(_) => panic!("Deveria ser um Filled"),
        };
    }

    pub fn clear(&mut self) {
        self.arr.clear();
        self.length = 0;
        self.last_empty = NULL_INDEX;
    }

    pub fn len(&self) -> i32 {
        self.length
    }

    pub fn get_node(&self, node: i32) -> Option<&T> {
        match self.arr.get(node as usize) {
            Some(node) => { node.try_get() },
            None => { None },
        }
    }

    pub fn get_mut_node(&mut self, node: i32) -> Option<&mut T> {
        match self.arr.get_mut(node as usize) {
            Some(node) => { node.try_get_mut() },
            None => { None },
        }
    }
}