use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use petgraph::{Direction, Graph, graph::NodeIndex, visit::EdgeRef};

/// Finds all Strongly Connected Components (SCCs) in a directed Graph
pub struct TarjanSCC<'a, NodeId, ConnectionId> {
    graph: &'a Graph<NodeId, ConnectionId>,
    current_index: usize,
    indices: HashMap<NodeIndex, usize>,
    lowlinks: HashMap<NodeIndex, usize>,
    stack: Vec<NodeIndex>,
    on_stack: HashSet<NodeIndex>,
    sccs: Vec<Vec<NodeIndex>>,
}

impl<'a, NodeId, ConnectionId> TarjanSCC<'a, NodeId, ConnectionId> {
    pub fn new(graph: &'a Graph<NodeId, ConnectionId>) -> Self {
        Self {
            graph,
            current_index: 0,
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            stack: Vec::new(),
            on_stack: HashSet::new(),
            sccs: Vec::new(),
        }
    }

    pub fn compute_sccs_as_indexes(&mut self) -> Vec<Vec<NodeIndex>> {
        // Clear any previous state
        self.current_index = 0;
        self.indices.clear();
        self.lowlinks.clear();
        self.stack.clear();
        self.on_stack.clear();
        self.sccs.clear();

        // Process all nodes in the graph
        for node_index in self.graph.node_indices() {
            if !self.indices.contains_key(&node_index) {
                self.compute_scc_for_node(node_index);
            }
        }

        core::mem::take(&mut self.sccs)
    }

    pub fn compute_sccs_as_node_ids(&mut self) -> Vec<Vec<&NodeId>> {
        let sccs = self.compute_sccs_as_indexes();
        sccs.into_iter()
            .map(|scc| {
                scc.into_iter()
                    .map(|node_idx| &self.graph[node_idx])
                    .collect()
            })
            .collect()
    }

    fn compute_scc_for_node(&mut self, node: NodeIndex) {
        self.indices.insert(node, self.current_index);
        self.lowlinks.insert(node, self.current_index);
        self.current_index += 1;
        self.stack.push(node);
        self.on_stack.insert(node);

        // Process all outgoing neighbors
        for edge in self.graph.edges_directed(node, Direction::Outgoing) {
            let neighbor = edge.target();

            if !self.indices.contains_key(&neighbor) {
                // Neighbor hasn't been visited - recurse
                self.compute_scc_for_node(neighbor);
                let neighbor_lowlink = self.lowlinks[&neighbor];
                let current_lowlink = self.lowlinks.get_mut(&node).unwrap();
                *current_lowlink = (*current_lowlink).min(neighbor_lowlink);
            } else if self.on_stack.contains(&neighbor) {
                // Neighbor is on stack - back edge found
                let neighbor_index = self.indices[&neighbor];
                let current_lowlink = self.lowlinks.get_mut(&node).unwrap();
                *current_lowlink = (*current_lowlink).min(neighbor_index);
            }
        }

        // If node is a root node, pop the stack and create an SCC
        if self.lowlinks[&node] == self.indices[&node] {
            let mut scc = Vec::new();
            loop {
                let w = self.stack.pop().unwrap();
                self.on_stack.remove(&w);
                scc.push(w);
                if w == node {
                    break;
                }
            }
            self.sccs.push(scc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Graph;

    #[test]
    fn test_simple_cycle() {
        let mut graph = Graph::new();
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");

        graph.add_edge(n1, n2, ());
        graph.add_edge(n2, n3, ());
        graph.add_edge(n3, n1, ());

        let mut tarjan = TarjanSCC::new(&graph);
        let sccs = tarjan.compute_sccs_as_indexes();

        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 3);
    }

    #[test]
    fn test_multiple_sccs() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let n3 = graph.add_node(3);
        let n4 = graph.add_node(4);
        let n5 = graph.add_node(5);

        // First SCC: 1 -> 2 -> 3 -> 1
        graph.add_edge(n1, n2, "edge1");
        graph.add_edge(n2, n3, "edge2");
        graph.add_edge(n3, n1, "edge3");

        // Second SCC: 4 -> 5 -> 4
        graph.add_edge(n4, n5, "edge4");
        graph.add_edge(n5, n4, "edge5");

        // Connection between SCCs
        graph.add_edge(n3, n4, "bridge");

        let mut tarjan = TarjanSCC::new(&graph);
        let sccs = tarjan.compute_sccs_as_indexes();

        assert_eq!(sccs.len(), 2);

        // Check that we have one SCC of size 3 and one of size 2
        let mut sizes: Vec<usize> = sccs.iter().map(|scc| scc.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![2, 3]);
    }

    #[test]
    fn test_isolated_nodes() {
        let mut graph = Graph::new();
        let _n1 = graph.add_node("isolated1");
        let _n2 = graph.add_node("isolated2");
        let n3 = graph.add_node("connected");
        let n4 = graph.add_node("connected2");

        // Only one edge
        graph.add_edge(n3, n4, ());

        let mut tarjan = TarjanSCC::new(&graph);
        let sccs = tarjan.compute_sccs_as_indexes();

        assert_eq!(sccs.len(), 4); // Each node is its own SCC
        for scc in &sccs {
            assert_eq!(scc.len(), 1);
        }
    }

    #[test]
    fn test_with_data() {
        let mut graph = Graph::new();
        let n1 = graph.add_node("Node A");
        let n2 = graph.add_node("Node B");

        graph.add_edge(n1, n2, "connection");
        graph.add_edge(n2, n1, "back_connection");

        let mut tarjan = TarjanSCC::new(&graph);
        let sccs_with_data = tarjan.compute_sccs_as_node_ids();

        assert_eq!(sccs_with_data.len(), 1);
        assert_eq!(sccs_with_data[0].len(), 2);

        let node_data: HashSet<&str> = sccs_with_data[0].iter().copied().map(|s| *s).collect();
        assert!(node_data.contains(&"Node A"));
        assert!(node_data.contains(&"Node B"));
    }
}
