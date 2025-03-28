use std::{collections::{HashSet, VecDeque}, hash::Hash, marker::PhantomData};

use super::{VecPool, NULL_INDEX};

pub trait GraphTraversal<T>
where
    T: Clone + Eq + Hash {
    /**
     * Função que irá ser chamada quando um node é visitado
     * next é um array vazio que poderá ser preenchido com os próximos vizinhos,
     * Mas também pode retornar outro array se quiser
    */
    fn visit(&mut self, node: T, iter_state: &mut GraphTraversalIterState<T>);
}

pub struct GraphTraversalIterState<T> 
{
    visited: HashSet<T>,
    to_explore: VecDeque<T>
}

impl<T> GraphTraversalIterState<T> 
where
    T: Clone + Eq + Hash
{
    pub fn add_neighbor(&mut self, neighbor: T) {
        if !self.visited.contains(&neighbor) {
            self.to_explore.push_back(neighbor);
        }
    }
}

/**
 * Travessia DFS ou BFS, irá chamar o método .visit() para cada nó visitado
 */
pub struct GraphTraversalIter<'a, T, G>
where
    T: Clone + Eq + Hash,
    G: GraphTraversal<T>,
{
    graph: &'a mut G,
    iter_state: GraphTraversalIterState<T>,
    depth_first: bool
}

impl <'a, T, G> GraphTraversalIter<'a, T, G> 
where
    T: Clone + Eq + Hash,
    G: GraphTraversal<T> 
{
    fn new(graph: &'a mut G, start: T, depth_first: bool) -> GraphTraversalIter<'a, T, G> {
        let mut to_explore = VecDeque::new();
        to_explore.push_back(start);

        GraphTraversalIter {
            graph: graph,
            iter_state: GraphTraversalIterState {
                visited: HashSet::new(),
                to_explore: to_explore
            },
            depth_first: depth_first
        }
    }

    pub fn depth_first(graph: &'a mut G, start: T) {
        GraphTraversalIter::new(graph, start, true).traversal();
    }

    pub fn breadth_first(graph: &'a mut G, start: T) {
        GraphTraversalIter::new(graph, start, false).traversal();
    }
    
    fn traversal(&mut self) {
        while let Some(node_index) = if self.depth_first { self.iter_state.to_explore.pop_back() } else { self.iter_state.to_explore.pop_front() } {
            // Insere na lista, e se já está visitado deve pular este nó
            if !self.iter_state.visited.insert(node_index.clone()) {
                continue;
            }
            
            self.graph.visit(node_index, &mut self.iter_state);
        }
    }
}

/*
Com Iterator não tem como fazer dum jeito que funcione bem, desisto

Ex:
for pos in GraphTraversalIter::breadth_first_iter(self, (gx, gy)) {
    // se o iterator fizer borrowed mutable não pode fazer nada com self aqui
    // se o iterator fizer borrow imutable pode só ler
}

sem usar .clone() é inviável usar o iterator*/
pub struct GraphTraversalIter2 { }
impl GraphTraversalIter2 {    
    fn traversal<G, T, CB>(graph: &mut G, start: T, depth_first: bool, mut callback: CB)
    where
        T: Clone + Eq + Hash,
        G: GraphTraversal<T>,
        CB: FnMut(&mut G, T)
    {
        let mut to_explore = VecDeque::new();
        to_explore.push_back(start);

        let mut iter_state = GraphTraversalIterState {
            visited: HashSet::new(),
            to_explore: to_explore
        };

        while let Some(node_index) = if depth_first { iter_state.to_explore.pop_back() } else { iter_state.to_explore.pop_front() } {
            // Insere na lista, e se já está visitado deve pular este nó
            if !iter_state.visited.insert(node_index.clone()) {
                continue;
            }
            
            graph.visit(node_index.clone(), &mut iter_state);
            (callback)(graph, node_index);
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
    fn add_vertex(&mut self, value: T) -> usize {
        self.vertices.alloc_node(GraphPoolNode { value: value, conns: Vec::new() })
    }

    fn connect(&mut self, a: usize, b: usize) {
        self.vertices[a].conns.push(b);
    }
}

impl<'a, T> GraphTraversal<usize> for GraphPoolNode<T> {
    fn visit(&mut self, node: usize, state: &mut GraphTraversalIterState<usize>) {
        for c in self.conns.iter() {
            state.add_neighbor(*c);
        }
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
            str: String,
            received: String
        }

        impl<'a> GraphTraversal<i32> for TestVisit {
            fn visit(&mut self, node: i32, iter_state: &mut GraphTraversalIterState<i32>) {
                // Test
                let mut chars = self.str.chars();
                let prev = if node > 0 { chars.nth((node-1) as usize) } else { None };
                let curr = chars.next();
                let next = chars.next();

                assert_ne!(curr, None);
                self.received.push(curr.unwrap());

                // Add neighbors
                if prev != Some(' ') && prev != None {
                    iter_state.add_neighbor(node-1);
                }
                if next != Some(' ') && next != None {
                    iter_state.add_neighbor(node+1);
                }
            }
        }

        {
            let mut test = TestVisit { str: String::from("jabuticaba   abc"), received: String::new() };
            let mut received = String::new();
            GraphTraversalIter2::traversal(&mut test, 0, true, |test, index| {
                received.push(test.str.chars().nth(index as usize).unwrap());
            });
            assert_eq!(test.received, String::from("jabuticaba"));
        }

        {
            let mut test = TestVisit { str: String::from("jabuticaba   abc"), received: String::new() };
            GraphTraversalIter::breadth_first(&mut test, 0);
            assert_eq!(test.received, String::from("jabuticaba"));
            
            test.received = String::new();
            GraphTraversalIter::breadth_first(&mut test, 9);
            assert_eq!(test.received, String::from("abacitubaj"));

            test.received = String::new();
            GraphTraversalIter::breadth_first(&mut test, 11);
            assert_eq!(test.received, String::from(" "));
            
            test.received = String::new();
            GraphTraversalIter::breadth_first(&mut test, 5);
            assert_eq!(test.received, String::from("itcuabbaaj"));
        }
        {
            let mut test = TestVisit { str: String::from("jabuticaba   abc"), received: String::new() };

            GraphTraversalIter::depth_first(&mut test, 0);
            assert_eq!(test.received, String::from("jabuticaba"));

            test.received = String::new();
            GraphTraversalIter::depth_first(&mut test, 9);
            assert_eq!(test.received, String::from("abacitubaj"));

            test.received = String::new();
            GraphTraversalIter::breadth_first(&mut test, 11);
            assert_eq!(test.received, String::from(" "));

            test.received = String::new();
            GraphTraversalIter::depth_first(&mut test, 5);
            assert_eq!(test.received, String::from("icabatubaj"));
        }
    }
}