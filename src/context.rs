use indexmap::IndexMap;
use oneormany::OneOrMany;
use serde::{Deserialize, Serialize};

use crate::terms;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct Context(pub OneOrMany<ContextItem>);

impl Context {
    pub fn ro_crate_1_1() -> Self {
        Context(OneOrMany::One(ContextItem::Reference(
            crate::constants::RO_CRATE_1_1_CONTEXT.to_string(),
        )))
    }

    pub fn ro_crate_1_2() -> Self {
        Context(OneOrMany::One(ContextItem::Reference(
            crate::constants::RO_CRATE_1_2_CONTEXT.to_string(),
        )))
    }

    pub fn ro_crate_1_3() -> Self {
        Context(OneOrMany::One(ContextItem::Reference(
            crate::constants::RO_CRATE_1_3_CONTEXT.to_string(),
        )))
    }

    pub fn new_from_iri(iri: impl Into<String>) -> Self {
        Context(OneOrMany::One(ContextItem::Reference(iri.into())))
    }

    pub fn items(&self) -> impl Iterator<Item = &ContextItem> {
        self.0.iter()
    }

    /// Whether `term` is usable in this crate
    pub fn defines(&self, term: &str) -> bool {
        self.items().any(|entry| match entry {
            ContextItem::Definitions(definitions) => definitions.contains_key(term),
            ContextItem::Reference(iri) => terms::context_defines(iri, term),
        })
    }

     /// The IRI `term` expands to, when the crate defines it inline.
    pub fn definition(&self, term: &str) -> Option<&str> {
        self.items().find_map(|entry| match entry {
            ContextItem::Definitions(definitions) => definitions.get(term).map(String::as_str),
            ContextItem::Reference(_) => None,
        })
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::ro_crate_1_3()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ContextItem {
    Reference(String),
    Definitions(IndexMap<String, String>),
}
