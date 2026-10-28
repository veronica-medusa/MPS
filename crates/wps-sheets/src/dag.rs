use crate::cell::{CellCoordinate, CellError, CellRange};
use petgraph::algo::tarjan_scc;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DependencyDag {
    graph: StableDiGraph<CellCoordinate, ()>,
    coord_to_node: HashMap<CellCoordinate, NodeIndex>,
    spill_ranges: HashMap<CellCoordinate, CellRange>,
}

impl DependencyDag {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_or_create_node(&mut self, coord: CellCoordinate) -> NodeIndex {
        if let Some(&node) = self.coord_to_node.get(&coord) {
            node
        } else {
            let node = self.graph.add_node(coord);
            self.coord_to_node.insert(coord, node);
            node
        }
    }

    pub fn set_dependencies(&mut self, target: CellCoordinate, dependencies: &[CellCoordinate]) {
        let target_node = self.get_or_create_node(target);

        // Remove incoming edges to target_node
        let incoming: Vec<NodeIndex> = self.graph.neighbors_directed(target_node, petgraph::Direction::Incoming).collect();
        for source in incoming {
            if let Some(edge) = self.graph.find_edge(source, target_node) {
                self.graph.remove_edge(edge);
            }
        }

        // Add new edges: dep -> target (target depends on dep)
        for &dep in dependencies {
            let dep_node = self.get_or_create_node(dep);
            self.graph.add_edge(dep_node, target_node, ());
        }
    }

    /// Checks for circular references using Tarjan's Strongly Connected Components
    pub fn has_cycle(&self) -> bool {
        let sccs = tarjan_scc(&self.graph);
        sccs.iter().any(|scc| scc.len() > 1 || (scc.len() == 1 && self.graph.contains_edge(scc[0], scc[0])))
    }

    /// Returns all cells in circular reference loops
    pub fn find_circular_cells(&self) -> Vec<CellCoordinate> {
        let sccs = tarjan_scc(&self.graph);
        let mut circular = Vec::new();
        for scc in sccs {
            if scc.len() > 1 || (scc.len() == 1 && self.graph.contains_edge(scc[0], scc[0])) {
                for node in scc {
                    if let Some(&coord) = self.graph.node_weight(node) {
                        circular.push(coord);
                    }
                }
            }
        }
        circular
    }

    /// Computes topological recalculation order for dirty cells and their transitive dependents
    pub fn get_recalculation_order(&self, dirty: &[CellCoordinate]) -> Result<Vec<CellCoordinate>, Vec<CellCoordinate>> {
        let circular = self.find_circular_cells();
        if !circular.is_empty() {
            return Err(circular);
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        for &d in dirty {
            if let Some(&node) = self.coord_to_node.get(&d) {
                queue.push_back(node);
                visited.insert(node);
            }
        }

        // BFS to collect all reachable dependents
        let mut reachable_nodes = Vec::new();
        while let Some(current) = queue.pop_front() {
            reachable_nodes.push(current);
            for neighbor in self.graph.neighbors_directed(current, petgraph::Direction::Outgoing) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        // Topologically sort reachable subgraph using Kahn's algorithm
        let mut in_degrees: HashMap<NodeIndex, usize> = HashMap::new();
        for &node in &reachable_nodes {
            let count = self.graph.neighbors_directed(node, petgraph::Direction::Incoming)
                .filter(|n| visited.contains(n))
                .count();
            in_degrees.insert(node, count);
        }

        let mut ready: std::collections::VecDeque<NodeIndex> = in_degrees.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&node, _)| node)
            .collect();

        let mut order = Vec::new();
        while let Some(node) = ready.pop_front() {
            if let Some(&coord) = self.graph.node_weight(node) {
                order.push(coord);
            }
            for neighbor in self.graph.neighbors_directed(node, petgraph::Direction::Outgoing) {
                if let Some(deg) = in_degrees.get_mut(&neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        ready.push_back(neighbor);
                    }
                }
            }
        }

        Ok(order)
    }

    /// Dynamic Array Spilling: records a spill range and detects collisions
    pub fn register_spill(
        &mut self,
        origin: CellCoordinate,
        rows: u32,
        cols: u16,
    ) -> Result<CellRange, CellError> {
        let spill_range = CellRange::new(
            origin,
            CellCoordinate::new(origin.row + rows - 1, origin.col + cols - 1),
        );

        // Check collision with existing spills
        for (&other_origin, existing_range) in &self.spill_ranges {
            if other_origin != origin && existing_range.contains(origin) {
                return Err(CellError::Spill);
            }
        }

        self.spill_ranges.insert(origin, spill_range);
        Ok(spill_range)
    }

    pub fn clear_spill(&mut self, origin: CellCoordinate) {
        self.spill_ranges.remove(&origin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_cycle_detection() {
        let mut dag = DependencyDag::new();
        let a1 = CellCoordinate::new(0, 0);
        let b1 = CellCoordinate::new(0, 1);

        // A1 = B1
        dag.set_dependencies(a1, &[b1]);
        assert!(!dag.has_cycle());

        // B1 = A1 (creates cycle A1 <-> B1)
        dag.set_dependencies(b1, &[a1]);
        assert!(dag.has_cycle());

        let circular = dag.find_circular_cells();
        assert_eq!(circular.len(), 2);
    }
}
