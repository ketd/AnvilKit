//! # Asset dependency tracking
//!
//! Tracks parent-child relationships between assets for cascade unloading.

use std::collections::{HashMap, HashSet};

/// Tracks parent→child dependency relationships between assets.
///
/// When a parent asset is removed, children with no remaining
/// parents are collected for cascade unloading.
pub struct DependencyGraph {
    /// asset → set of assets it depends on (children)
    deps: HashMap<u64, HashSet<u64>>,
    /// asset → set of assets that depend on it (parents)
    reverse: HashMap<u64, HashSet<u64>>,
}

// Use a static empty set for returning references when key not found
static EMPTY: std::sync::LazyLock<HashSet<u64>> = std::sync::LazyLock::new(HashSet::new);

impl DependencyGraph {
    pub fn new() -> Self {
        Self { deps: HashMap::new(), reverse: HashMap::new() }
    }

    /// Register that `parent` depends on `child`.
    pub fn add_dependency(&mut self, parent: u64, child: u64) {
        self.deps.entry(parent).or_default().insert(child);
        self.reverse.entry(child).or_default().insert(parent);
    }

    /// Remove a dependency edge.
    pub fn remove_dependency(&mut self, parent: u64, child: u64) {
        if let Some(set) = self.deps.get_mut(&parent) {
            set.remove(&child);
            if set.is_empty() { self.deps.remove(&parent); }
        }
        if let Some(set) = self.reverse.get_mut(&child) {
            set.remove(&parent);
            if set.is_empty() { self.reverse.remove(&child); }
        }
    }

    /// Direct dependencies of an asset.
    pub fn dependencies_of(&self, id: u64) -> &HashSet<u64> {
        self.deps.get(&id).unwrap_or(&EMPTY)
    }

    /// Assets that depend on this asset.
    pub fn dependents_of(&self, id: u64) -> &HashSet<u64> {
        self.reverse.get(&id).unwrap_or(&EMPTY)
    }

    /// Check if any other asset depends on this one.
    pub fn has_dependents(&self, id: u64) -> bool {
        self.reverse.get(&id).map_or(false, |s| !s.is_empty())
    }

    /// Remove an asset without cascading.
    pub fn remove(&mut self, id: u64) {
        // Remove all edges where id is a parent
        if let Some(children) = self.deps.remove(&id) {
            for child in &children {
                if let Some(parents) = self.reverse.get_mut(child) {
                    parents.remove(&id);
                    if parents.is_empty() { self.reverse.remove(child); }
                }
            }
        }
        // Remove all edges where id is a child
        if let Some(parents) = self.reverse.remove(&id) {
            for parent in &parents {
                if let Some(children) = self.deps.get_mut(parent) {
                    children.remove(&id);
                    if children.is_empty() { self.deps.remove(parent); }
                }
            }
        }
    }

    /// Remove an asset and cascade: collect orphaned children recursively.
    /// Returns IDs that should also be unloaded (excludes `id` itself).
    pub fn remove_and_cascade(&mut self, id: u64) -> Vec<u64> {
        let children: Vec<u64> = self.deps.get(&id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();

        self.remove(id);

        let mut cascade = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        for child in children {
            if !self.has_dependents(child) {
                queue.push_back(child);
            }
        }

        while let Some(orphan) = queue.pop_front() {
            let grandchildren: Vec<u64> = self.deps.get(&orphan)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();

            self.remove(orphan);
            cascade.push(orphan);

            for gc in grandchildren {
                if !self.has_dependents(gc) {
                    queue.push_back(gc);
                }
            }
        }

        cascade
    }

    pub fn len(&self) -> usize {
        let mut all = HashSet::new();
        for (k, v) in &self.deps {
            all.insert(*k);
            all.extend(v);
        }
        all.len()
    }

    pub fn is_empty(&self) -> bool {
        self.deps.is_empty() && self.reverse.is_empty()
    }

    pub fn clear(&mut self) {
        self.deps.clear();
        self.reverse.clear();
    }
}

impl Default for DependencyGraph {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_query() {
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.add_dependency(1, 3);
        assert!(g.dependencies_of(1).contains(&2));
        assert!(g.dependencies_of(1).contains(&3));
        assert!(g.dependents_of(2).contains(&1));
        assert!(g.has_dependents(2));
        assert!(!g.has_dependents(1));
    }

    #[test]
    fn remove_dependency_edge() {
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.remove_dependency(1, 2);
        assert!(g.dependencies_of(1).is_empty());
        assert!(!g.has_dependents(2));
        assert!(g.is_empty());
    }

    #[test]
    fn cascade_simple() {
        // A -> B -> C
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.add_dependency(2, 3);
        let cascaded = g.remove_and_cascade(1);
        // B has no other parents → cascaded, then C has no parents → cascaded
        assert!(cascaded.contains(&2));
        assert!(cascaded.contains(&3));
        assert!(g.is_empty());
    }

    #[test]
    fn shared_dep_no_cascade() {
        // A -> C, B -> C. Remove A: C still has parent B.
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 3);
        g.add_dependency(2, 3);
        let cascaded = g.remove_and_cascade(1);
        assert!(cascaded.is_empty());
        assert!(g.has_dependents(3)); // B still depends on C
    }

    #[test]
    fn diamond_cascade() {
        // A -> B, A -> C, B -> D, C -> D
        // Remove A: B and C have no parents → cascade.
        // Then D has no parents → cascade.
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.add_dependency(1, 3);
        g.add_dependency(2, 4);
        g.add_dependency(3, 4);
        let cascaded = g.remove_and_cascade(1);
        assert!(cascaded.contains(&2));
        assert!(cascaded.contains(&3));
        assert!(cascaded.contains(&4));
        assert!(g.is_empty());
    }

    #[test]
    fn diamond_partial_cascade() {
        // A -> B, A -> C, B -> D, C -> D, E -> D
        // Remove A: B,C cascade. D still has parent E.
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.add_dependency(1, 3);
        g.add_dependency(2, 4);
        g.add_dependency(3, 4);
        g.add_dependency(5, 4);
        let cascaded = g.remove_and_cascade(1);
        assert!(cascaded.contains(&2));
        assert!(cascaded.contains(&3));
        assert!(!cascaded.contains(&4)); // E still holds D
        assert!(g.has_dependents(4));
    }

    #[test]
    fn remove_without_cascade() {
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.remove(1);
        assert!(!g.has_dependents(2));
    }

    #[test]
    fn empty_graph() {
        let g = DependencyGraph::new();
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
        assert!(g.dependencies_of(999).is_empty());
        assert!(g.dependents_of(999).is_empty());
    }

    #[test]
    fn clear() {
        let mut g = DependencyGraph::new();
        g.add_dependency(1, 2);
        g.add_dependency(3, 4);
        g.clear();
        assert!(g.is_empty());
    }
}
