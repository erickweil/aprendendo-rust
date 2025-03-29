use std::{collections::{HashSet, VecDeque}, hash::Hash, marker::PhantomData};

use super::{LinkedStack, Stack, VecPool, NULL_INDEX};

pub trait GraphIterState<T> {
    fn push_neighbor(&mut self, neighbor: T);
}

pub struct GraphIter<T> 
{
    visited: HashSet<T>,
    to_explore: VecDeque<T>,
    depth_first: bool
}

impl<T> GraphIter<T> 
where
    T: Clone + Eq + Hash
{
    pub fn depth_first(start: T) -> Self {
        return Self::new(true, start);
    }

    pub fn breadth_first(start: T) -> Self {
        return Self::new(false, start);
    }

    fn new(depth_first: bool, start: T) -> Self {
        let mut to_explore = VecDeque::new();
        to_explore.push_back(start);

        Self {
            visited: HashSet::new(),
            to_explore: to_explore,
            depth_first: depth_first
        }
    }

    pub fn next(&mut self) -> Option<T> {
        while let Some(node_index) = if self.depth_first { 
            self.to_explore.pop_back() 
        } else { 
            self.to_explore.pop_front() 
        } {
            // Insere na lista, e se já está visitado deve pular este nó
            if !self.visited.insert(node_index.clone()) {
                continue;
            }

            return Some(node_index);
        }
        return None;
    }
}

impl<T> GraphIterState<T> for GraphIter<T> 
where
    T: Clone + Eq + Hash
{  
    fn push_neighbor(&mut self, neighbor: T) {
        if !self.visited.contains(&neighbor) {
            self.to_explore.push_back(neighbor);
        }
    }
}

// A FAZER: medir se faz sentido substituir no to_explore Vec<T> por um tipo de push-only immutable Single Linked List, que seria leve de 'copiar'
// Já que não seriam cópias reais e sim referências ao resto da lista, diminuiria muito uso de memória para buscas grandes
pub struct GraphSearch<T> 
{
    visited: HashSet<T>,
    // Utilizando LinkedStack para ser econômico na memória copiar os caminhos
    to_explore: VecDeque<LinkedStack<T>>,
    current_path: Option<LinkedStack<T>>,
    depth_first: bool
}

impl<T> GraphSearch<T> 
where
    T: Clone + Eq + Hash
{
    pub fn depth_first(start: T) -> Self {
        return Self::new(true, start);
    }

    pub fn breadth_first(start: T) -> Self {
        return Self::new(false, start);
    }

    fn new(depth_first: bool, start: T) -> Self {
        let mut first_path = LinkedStack::new();
        first_path.push(start);

        Self {
            visited: HashSet::new(),
            to_explore: VecDeque::from([first_path]),
            current_path: None,
            depth_first: depth_first
        }
    }

    pub fn next(&mut self) -> Option<T> {
        while let Some(path) = if self.depth_first { 
            self.to_explore.pop_back() 
        } else { 
            self.to_explore.pop_front() 
        } {
            let node_index = path.peek().unwrap().clone();
            // Insere na lista, e se já está visitado deve pular este nó
            if !self.visited.insert(node_index.clone()) {
                continue;
            }

            self.current_path = Some(path);
            return Some(node_index);
        }
        return None;
    }

    /** Deve chamar somente após encerrar iteração */
    pub fn get_path(self) -> LinkedStack<T> {
        return self.current_path.unwrap();
    }
}

impl<T> GraphIterState<T> for GraphSearch<T> 
where
    T: Clone + Eq + Hash
{  
    fn push_neighbor(&mut self, neighbor: T) {
        if !self.visited.contains(&neighbor) {
            let mut path = self.current_path.as_ref().unwrap().clone();
            path.push(neighbor);

            self.to_explore.push_back(path);
        }
    }
}


// Falta decidir como usar melhor isso
pub struct GraphPool<T> {
    vertices: VecPool<GraphPoolNode<T>>
}

struct GraphPoolNode<T> {
    value: T,
    conns: Vec<usize>
}

impl<T> GraphPool<T> {
    fn new() -> GraphPool<T> {
        GraphPool { vertices: VecPool::new() }
    }

    fn get(&self, node_index: usize) -> &T {
        return &self.vertices[node_index].value;
    }

    fn add_vertex(&mut self, value: T) -> usize {
        self.vertices.alloc_node(GraphPoolNode { value: value, conns: Vec::new() })
    }

    fn connect(&mut self, a: usize, b: usize) {
        self.vertices[a].conns.push(b);
    }

    fn connect_bi(&mut self, a: usize, b: usize) {
        self.vertices[a].conns.push(b);
        self.vertices[b].conns.push(a);
    }

    fn visit<S: GraphIterState<usize>>(&self, node_index: usize, iter_state: &mut S) -> &T {
        let node = &self.vertices[node_index];
        for neighbor in node.conns.iter() {
            iter_state.push_neighbor(*neighbor);
        }
        return &node.value;
    }
}

#[cfg(test)]
mod test {
    use std::ops::Index;

    use super::*;

    #[test]
    pub fn graph_iter() {
        // Simulando grafo com string
        struct TestVisit {
            str: String
        }

        impl TestVisit {
            fn push_neighbor<T: GraphIterState<i32>>(&self, node: i32, iter_state: &mut T) -> char {
                // Test
                let mut chars = self.str.chars();
                let prev = if node > 0 { chars.nth((node-1) as usize) } else { None };
                let curr = chars.next();
                let next = chars.next();

                // Add neighbors
                if prev != Some(' ') && prev != None {
                    iter_state.push_neighbor(node-1);
                }
                if next != Some(' ') && next != None {
                    iter_state.push_neighbor(node+1);
                }

                return curr.unwrap();
            }

            fn run_iter(str: &str, depth_first: bool, start: i32) -> String {
                let test = TestVisit { str: String::from(str) };
                let mut received = String::new();

                let mut iter_state = GraphIter::new(depth_first, start);
                while let Some(node_index) = iter_state.next() {            
                    let c = test.push_neighbor(node_index, &mut iter_state);
                    received.push(c);
                }

                return received;
            }
        }

        assert_eq!(TestVisit::run_iter("jabuticaba   abc", false, 0),  String::from("jabuticaba"));
        assert_eq!(TestVisit::run_iter("jabuticaba   abc", false, 9),  String::from("abacitubaj"));
        assert_eq!(TestVisit::run_iter("jabuticaba   abc", false, 11), String::from(" "));
        assert_eq!(TestVisit::run_iter("jabuticaba   abc", false, 5),  String::from("itcuabbaaj"));

        assert_eq!(TestVisit::run_iter("jabuticaba   abc", true, 0),  String::from("jabuticaba"));
        assert_eq!(TestVisit::run_iter("jabuticaba   abc", true, 9),  String::from("abacitubaj"));
        assert_eq!(TestVisit::run_iter("jabuticaba   abc", true, 11), String::from(" "));
        assert_eq!(TestVisit::run_iter("jabuticaba   abc", true, 5),  String::from("icabatubaj"));
    }

    #[test]
    pub fn graph_struct() {
        let mut graph: GraphPool<char> = GraphPool::new();

        /**
         *   E --- A
         *   |     |
         *   F     B*
         *    \  /   \
         *     C ---- D    
         */
        let a = graph.add_vertex('A');
        let b = graph.add_vertex('B');
        let c = graph.add_vertex('C');
        let d = graph.add_vertex('D');
        let e = graph.add_vertex('E');
        let f = graph.add_vertex('F');

        graph.connect_bi(a, b);
        graph.connect_bi(a, e);

        graph.connect_bi(b, d);
        graph.connect_bi(b, c);

        graph.connect_bi(c, d);
        graph.connect_bi(c, f);

        graph.connect_bi(f, e);

        // Traversal only
        {
            let mut visited = String::new();
            let mut iter_state = GraphIter::breadth_first(b);
            while let Some(node_index) = iter_state.next() {            
                let c = graph.visit(node_index, &mut iter_state);
                
                visited.push(*c);
            }

            assert_eq!(visited, String::from("BADCEF"));
        }

        // Search
        {
            let mut visited = String::new();
            let mut iter_state = GraphSearch::breadth_first(b);
            while let Some(node_index) = iter_state.next() {            
                let c = graph.visit(node_index, &mut iter_state);
                
                visited.push(*c);
            }
            assert_eq!(visited, String::from("BADCEF"));


            visited = String::new();
            let path = iter_state.get_path();
            for node_index in path.iter() {
                visited.push(graph.get(*node_index).clone());
            }
            assert_eq!(visited, String::from("FCB")); // porque é pilha é ao contrário
        }
    }
}