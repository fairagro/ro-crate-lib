use crate::{
    constants::{METADATA_FILE_NAMES, ROOT_ID},
    context::Context,
    graph::{Graph, node::GraphNode},
    profile::Profile,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub mod build;
pub mod constants;
pub mod context;
pub mod graph;
pub mod profile;
pub mod terms;
pub mod validate;
pub mod views;

/// Parsed representation of ro-crate-metadata.json
///
/// Parsing is deliberately lenient
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RoCrate {
    #[serde(rename = "@context")]
    pub context: Context,
    #[serde(rename = "@graph")]
    pub graph: Graph,
}

impl RoCrate {
    /// The metadata descriptor: the `CreativeWork` that is `about` the root.
    pub fn descriptor(&self) -> Option<&GraphNode> {
        self.graph
            .iter()
            .find(|node| is_metadata_id(&node.id) && node.get("about").is_some())
            .or_else(|| self.graph.iter().find(|node| is_metadata_id(&node.id)))
    }

    /// The root data entity, via the descriptor's `about`.
    pub fn root(&self) -> Option<&GraphNode> {
        self.descriptor()
            .and_then(|descriptor| descriptor.iris("about").next())
            .and_then(|id| self.graph.get(id))
            .or_else(|| self.graph.get(ROOT_ID))
    }

    pub fn root_mut(&mut self) -> Option<&mut GraphNode> {
        let id = self.root()?.id.clone();
        self.graph.get_mut(&id)
    }

    /// The root's `mainEntity` — the main workflow, in a Workflow RO-Crate.
    pub fn main_entity(&self) -> Option<&GraphNode> {
        let id = self.root()?.iris("mainEntity").next()?;
        self.graph.get(id)
    }

    /// Every `conformsTo` IRI, from the descriptor and from the root.
    ///
    /// The base spec version sits on the descriptor; the Workflow Run profiles
    /// sit on the root, and published crates put Workflow RO-Crate on either.
    pub fn conforms_to(&self) -> impl Iterator<Item = &str> {
        self.descriptor()
            .into_iter()
            .chain(self.root())
            .flat_map(|node| node.iris("conformsTo"))
    }

    /// Claimed profiles, in the order they appear, without duplicates.
    pub fn profiles(&self) -> Vec<Profile> {
        let mut profiles = Vec::new();
        for profile in self.conforms_to().map(Profile::from_iri) {
            if !profiles.contains(&profile) {
                profiles.push(profile);
            }
        }
        profiles
    }

    /// Whether the crate claims `profile`, at any version.
    pub fn claims(&self, profile: &Profile) -> bool {
        self.profiles().iter().any(|p| p.is_same_profile(profile))
    }

    /// The entities reachable from the root by `hasPart`, in breadth-first
    /// order. Nested `Dataset` parts are followed.
    pub fn data_entities(&self) -> Vec<&GraphNode> {
        let Some(root) = self.root() else {
            return Vec::new();
        };

        let mut found: Vec<&GraphNode> = Vec::new();
        let mut queue: VecDeque<&str> = root.iris("hasPart").collect();
        while let Some(id) = queue.pop_front() {
            if id == root.id || found.iter().any(|node| node.id == id) {
                continue;
            }
            let Some(node) = self.graph.get(id) else {
                continue;
            };
            found.push(node);
            queue.extend(node.iris("hasPart"));
        }
        found
    }

    /// Everything that is neither the root, the descriptor, nor a data entity.
    pub fn contextual_entities(&self) -> Vec<&GraphNode> {
        let data = self.data_entities();
        let is_root_or_descriptor = |node: &GraphNode| {
            [self.root(), self.descriptor()]
                .iter()
                .flatten()
                .any(|special| special.id == node.id)
        };

        self.graph
            .iter()
            .filter(|node| !is_root_or_descriptor(node))
            .filter(|node| !data.iter().any(|entity| entity.id == node.id))
            .collect()
    }
}

/// `ro-crate-metadata.json`, also as the last segment of an absolute IRI.
fn is_metadata_id(id: &str) -> bool {
    let name = id.rsplit('/').next().unwrap_or(id);
    METADATA_FILE_NAMES.contains(&name)
}
