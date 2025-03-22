use std::mem;

use rand::distr::slice::Empty;

// valor de índice que será utilizado para representar a ausência de conexão
pub const NULL_INDEX: usize = usize::MAX;

#[derive(Debug)]
enum IndexNode<T> {
    Filled(T),
    Empty(usize)
}

impl<T> IndexNode<T> {
    /** Não poderá usar o valor depois de chamar, retorna o ownership */
    fn get_owned(self) -> Option<T> {
        match self {
            IndexNode::Filled(value) => Some(value),
            IndexNode::Empty(_) => None,
        }
    }

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
    length: usize,
    last_empty: usize
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
    pub fn alloc_node(&mut self, value: T) -> usize {
        let node = IndexNode::Filled(value);
        if self.last_empty == NULL_INDEX { // não tem nenhum vazio, adicionar mais um no final do array
            self.arr.push(node);
            self.length += 1;

            return self.arr.len()-1;
        } else {
            // fazer pop() da pilha de valores vazios
            // 1 - obter índice do espaço vazio
            let free_node = self.last_empty;

            // 2 - topo da pilha deve apontar para o próximo espaço vazio (ou NULL)
            if let IndexNode::Empty(next) = self.arr[self.last_empty] {
                self.last_empty = next;
            } else {
                panic!("NUNCA DEVERIA OCORRER: Ao obter o last_empty obteve um Filled");
            }

            // Re-Inicializar os valores
            self.arr[free_node] = node;
            self.length += 1;

            return free_node;
        }
    }

    /**
     * Libera um nó da lista tornando um espaço vazio disponível, e retorna o valor que estava nele,
     */
    pub fn free_node(&mut self, node: usize) -> Option<T> {
        let len = self.arr.len();
        if let Some(arr_node) = self.arr.get_mut(node) {
            if let IndexNode::Empty(_) = arr_node {
                return None; // free em nó Empty (double free?)
            }
            
            let ret_value: Option<T>;
            if node != len - 1 {
                // Se removeu e não é no fim do array, marca como vazio
                // - fazer push() na pilha de valores vazios
                let ret = mem::replace(arr_node, IndexNode::Empty(self.last_empty));
                self.last_empty = node;
                self.length -= 1;

                ret_value = ret.get_owned();
            } else {
                // Removendo do fim do array, basta fazer pop
                let ret = self.arr.pop().unwrap();
                self.length -= 1;

                ret_value = ret.get_owned();
            }

            if(self.length == 0) {
                // Se ficou vazio, limpar os empty
                self.clear();
            }

            ret_value
        } else {
            None // free em índice fora do array
        }
    }

    pub fn clear(&mut self) {
        self.arr.clear();
        self.length = 0;
        self.last_empty = NULL_INDEX;
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get_node(&self, node: usize) -> Option<&T> {
        match self.arr.get(node) {
            Some(node) => { node.try_get() },
            None => { None },
        }
    }

    pub fn get_mut_node(&mut self, node: usize) -> Option<&mut T> {
        match self.arr.get_mut(node) {
            Some(node) => { node.try_get_mut() },
            None => { None },
        }
    }
}

#[cfg(test)]
mod test {
    use super::VecPool;


    #[test]
    pub fn arr_alloc_logic() {
        let mut pool = VecPool::new();
        let a = pool.alloc_node('?');
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.arr.len(), 1);

        assert_eq!(pool.get_node(a), Some(&'?'));
        let mut mut_a = pool.get_mut_node(a);
        assert_eq!(mut_a, Some(&mut '?'));
        *mut_a.unwrap() = 'A';

        for i in 0..5 {
            let b = pool.alloc_node('B');
            let c = pool.alloc_node('C');
            assert_eq!(pool.len(), 3);
            assert_eq!(pool.arr.len(), 3);

            assert_eq!(pool.get_node(b), Some(&'B'));
            assert_eq!(pool.get_node(c), Some(&'C'));

            assert_eq!(pool.free_node(c), Some('C'));
            assert_eq!(pool.free_node(b), Some('B'));
            // Se alocar B,C e liberar C,B irá fazer pop() array diminiu
            assert_eq!(pool.len(), 1);
            assert_eq!(pool.arr.len(), 1);

            assert_eq!(pool.get_node(b), None);
            assert_eq!(pool.get_node(c), None);
        }
        for i in 0..5 {
            let d = pool.alloc_node('D');
            let e = pool.alloc_node('E');
            assert_eq!(pool.len(), 3);
            assert_eq!(pool.arr.len(), 3);

            assert_eq!(pool.free_node(d), Some('D'));
            assert_eq!(pool.free_node(e), Some('E'));
            // Se alocar D,E e liberar D,E irá fazer pop() só para E, então array não diminiu
            assert_eq!(pool.len(), 1);
            assert_eq!(pool.arr.len(), 2);
        }
        assert_eq!(pool.free_node(a), Some('A'));
        assert_eq!(pool.len(), 0);
        assert_eq!(pool.arr.len(), 0); // 0 porque fez clear ao remover o último
        assert_eq!(pool.get_node(a), None);

        assert_eq!(pool.free_node(a), None);
        assert_eq!(pool.get_node(a), None);
    }
}