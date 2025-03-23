use std::{collections::{HashSet, VecDeque}, hash::Hash, marker::PhantomData};

use super::{VecPool, NULL_INDEX};

pub trait GraphTraversal<T> {
    /**
     * Função que irá ser chamada quando um node é visitado
     * next é um array vazio que poderá ser preenchido com os próximos vizinhos,
     * Mas também pode retornar outro array se quiser
    */
    fn visit<'a>(&mut self, node: T, neighbors: &'a mut Vec<T>) -> &'a mut Vec<T> {
        todo!()
    }

    fn get_neighbors<'a>(&self, node: T, neighbors: &'a mut Vec<T>) -> &'a mut Vec<T> {
        todo!()
    }
}

/**
 * Travessia DFS ou BFS, irá chamar o método .visit() para cada nó visitado
 */
pub struct GraphTraversalIter<'a,T : Copy + Eq + Hash> {
    graph: &'a mut dyn GraphTraversal<T>,
    visited: HashSet<T>,
    to_explore: VecDeque<T>,
    neighbors_cache: Vec<T>
}

impl<'a, T : Copy + Eq + Hash> GraphTraversalIter<'a, T> {
    fn new(graph: &'a mut dyn GraphTraversal<T>, start: T) -> GraphTraversalIter<'a, T> {
        let mut to_explore = VecDeque::new();
        to_explore.push_back(start);

        GraphTraversalIter {
            graph: graph,
            visited: HashSet::new(),
            to_explore: to_explore,
            neighbors_cache: Vec::new()
        }
    }

    pub fn depth_first(graph: &'a mut dyn GraphTraversal<T>, start: T) {
        GraphTraversalIter::new(graph, start).traversal(true);
    }

    pub fn breadth_first(graph: &'a mut dyn GraphTraversal<T>, start: T) {
        GraphTraversalIter::new(graph, start).traversal(false);
    }

    pub fn breadth_first_iter(graph: &'a mut dyn GraphTraversal<T>, start: T) -> Self {
        GraphTraversalIter::new(graph, start)
    }

    fn traversal(&mut self, depth_first: bool) {
        while let Some(node_index) = if depth_first { self.to_explore.pop_back() } else { self.to_explore.pop_front() } {
            // Insere na lista, e se já esá visitado deve pular este nó
            if !self.visited.insert(node_index) {
                continue;        
            }
            
            self.neighbors_cache.clear();
            let neighbors = self.graph.visit(node_index, &mut self.neighbors_cache);
            for &neighbor in neighbors.iter() {
                if !self.visited.contains(&neighbor) {
                    self.to_explore.push_back(neighbor);
                }
            }
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

sem usar .clone() é inviável usar o iterator

impl<'a, T: Copy + Eq + Hash> Iterator for GraphTraversalIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let depth_first = false;
        while let Some(node_index) = if depth_first { self.to_explore.pop_back() } else { self.to_explore.pop_front() } {
            if !self.visited.insert(node_index) {
                continue;
            }

            // Limpa e reutiliza o buffer
            self.neighbors_cache.clear();
            let neighbors = self.graph.get_neighbors(node_index, &mut self.neighbors_cache);
            for &neighbor in neighbors.iter() {
                if !self.visited.contains(&neighbor) { // Marca como visitado já na inserção
                    self.to_explore.push_back(neighbor);
                }
            }

            return Some(node_index);
        }
        None
    }
}
*/


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

impl<T> GraphTraversal<usize> for GraphPool<T> {
    fn visit<'a>(&mut self, node: usize, neighbors: &'a mut Vec<usize>) -> &'a mut Vec<usize> {
        return neighbors;
    }
}