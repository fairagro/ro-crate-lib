use crate::graph::node::{GraphNode, Reference};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod node;

#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    nodes: Vec<GraphNode>,
    index: HashMap<String, usize>,
}

impl Graph {
    fn reindex(&mut self) {
        self.index.clear();
        for (position, node) in self.nodes.iter().enumerate() {
            self.index.entry(node.id.clone()).or_insert(position);
        }
    }

    pub fn insert(&mut self, node: GraphNode) -> Option<GraphNode> {
        if let Some(&position) = self.index.get(&node.id) { Some(std::mem::replace(&mut self.nodes[position], node)) } else {
            self.index.insert(node.id.clone(), self.nodes.len());
            self.nodes.push(node);
            None
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<GraphNode> {
        let position = self.index.get(id).copied()?;
        let removed = self.nodes.remove(position);
        self.reindex();
        Some(removed)
    }

    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.index.contains_key(id)
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&GraphNode> {
        self.index.get(id).map(|&position| &self.nodes[position])
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut GraphNode> {
        let position = *self.index.get(id)?;
        self.nodes.get_mut(position)
    }

    #[must_use]
    pub fn resolve_reference(&self, reference: &Reference) -> Option<&GraphNode> {
        self.get(&reference.id)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, GraphNode> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, GraphNode> {
        self.nodes.iter_mut()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// `@id`s appearing more than once, in document order.
    #[must_use]
    pub fn duplicate_ids(&self) -> Vec<&str> {
        let mut seen = HashMap::new();
        let mut duplicates = Vec::new();
        for node in &self.nodes {
            let count = seen.entry(node.id.as_str()).or_insert(0usize);
            *count += 1;
            if *count == 2 {
                duplicates.push(node.id.as_str());
            }
        }
        duplicates
    }
}

impl Iterator for Graph {
    type Item = GraphNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.pop()
    }
}

impl FromIterator<GraphNode> for Graph {
    fn from_iter<T: IntoIterator<Item = GraphNode>>(iter: T) -> Self {
        Graph::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<'a> IntoIterator for &'a Graph {
    type Item = &'a GraphNode;
    type IntoIter = std::slice::Iter<'a, GraphNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<Vec<GraphNode>> for Graph {
    fn from(nodes: Vec<GraphNode>) -> Self {
        let mut graph = Graph {
            nodes,
            index: HashMap::new(),
        };

        graph.reindex();
        graph
    }
}

impl Serialize for Graph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.nodes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Graph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Graph::from(Vec::<GraphNode>::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_of(ids: &[&str]) -> Graph {
        ids.iter()
            .map(|id| GraphNode::new(*id, "Thing"))
            .collect::<Graph>()
    }

    #[test]
    fn insert_replaces_in_place() {
        let mut graph = graph_of(&["a", "b", "c"]);
        let mut replacement = GraphNode::new("b", "Dataset");
        replacement.set("name", "renamed");
        graph.insert(replacement);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph.iter().nth(1).unwrap().text("name"), Some("renamed"));
    }

    #[test]
    fn remove_keeps_the_index_correct() {
        let mut graph = graph_of(&["a", "b", "c"]);
        graph.remove("a");

        assert!(!graph.contains("a"));
        assert_eq!(graph.get("c").map(|e| e.id.as_str()), Some("c"));
    }

    #[test]
    fn duplicates_are_kept_and_reported() {
        let graph = graph_of(&["a", "a", "b"]);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph.duplicate_ids(), ["a"]);
    }
}
