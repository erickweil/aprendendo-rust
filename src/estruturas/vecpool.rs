use std::{mem, ops::{Index, IndexMut}};

use rand::distr::slice::Empty;

// valor de índice que será utilizado para representar a ausência de conexão
pub const NULL_INDEX: usize = usize::MAX;

#[derive(Debug, Clone)]
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
 * 
 * Referências/Trabalho similar:
 * - Slotmap (https://github.com/orlp/slotmap)
 *   - https://www.reddit.com/r/rust/comments/8zkedd/slotmap_a_new_crate_for_storing_values_with/
 *   - 4 minute version presented as a lightning talk at C++Now: https://www.youtube.com/watch?v=SHaAR7XPtNU
 *   - 30 minute version (includes other containers as well) presented as a CppCon session: https://www.youtube.com/watch?v=-8UZhDjgeZU
 * - Slab (https://github.com/tokio-rs/slab)
 * - https://www.reddit.com/r/rust/comments/gfo1uw/benchmarking_slotmap_slab_stable_vec_etc/
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
            let free_node_index = self.last_empty;

            let free_node = &mut self.arr[free_node_index];
            if let &mut IndexNode::Empty(next) = free_node {
                self.last_empty = next;

                // Re-Inicializar os valores
                *free_node = node;
                self.length += 1;

                return free_node_index;
            } else {
                panic!("NUNCA DEVERIA OCORRER: Ao obter o last_empty obteve um Filled");
            }
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

            // Se removeu e não é no fim do array, marca como vazio
            // - fazer push() na pilha de valores vazios
            let ret = mem::replace(arr_node, IndexNode::Empty(self.last_empty));
            self.last_empty = node;
            self.length -= 1;

            ret.get_owned()
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


// let value = pool[index];
impl<T> Index<usize> for VecPool<T> {
    type Output = T;

    fn index(&self, node: usize) -> &Self::Output {
        match &self.arr[node] {
            IndexNode::Filled(value) => value,
            IndexNode::Empty(_) => panic!("Acessou um Empty {:?}", node),
        }
    }
}

// pool[index] = value;
impl<T> IndexMut<usize> for VecPool<T> {
    fn index_mut(&mut self, node: usize) -> &mut Self::Output {
        match &mut self.arr[node] {
            IndexNode::Filled(value) => value,
            IndexNode::Empty(_) => panic!("Acessou um Empty {:?}", node),            
        }
    }
}

impl<T: Clone> Clone for VecPool<T> {
    fn clone(&self) -> Self {
        VecPool { 
            arr: self.arr.clone(), 
            length: self.length, 
            last_empty: self.last_empty 
        }
    }
}

#[cfg(test)]
mod test {
    use super::VecPool;
    use std::fmt::Write;

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
            // tamanho do pool diminui mas do array não
            assert_eq!(pool.len(), 1);
            assert_eq!(pool.arr.len(), 3);

            assert_eq!(pool.get_node(b), None);
            assert_eq!(pool.get_node(c), None);
        }
        assert_eq!(pool.free_node(a), Some('A'));
        assert_eq!(pool.len(), 0);
        assert_eq!(pool.arr.len(), 3);
        assert_eq!(pool.get_node(a), None);

        assert_eq!(pool.free_node(a), None);
        assert_eq!(pool.get_node(a), None);

        pool.clear();
        assert_eq!(pool.arr.len(), 0);  // 0 porque fez clear ao remover o último
    }


    #[test]
    pub fn index_operator() {
        let mut pool: VecPool<&str> = VecPool::new();

        let nome = pool.alloc_node("João");
        let sobrenome = pool.alloc_node("");
        
        pool[sobrenome] = "Silva";

        let mut f = String::new();
        write!(f,"{} {}", pool[nome], pool[sobrenome]);
        assert_eq!(f, "João Silva");
    }
}