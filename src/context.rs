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

    /// Makes `term` usable, by referencing the context document that defines
    /// it. Does nothing when the crate can already use the term, and returns
    /// whether the context grew.
    pub fn ensure(&mut self, term: &str) -> bool {
        if self.defines(term) {
            return false;
        }
        match terms::defining_context(term) {
            Some(iri) => self.reference(iri),
            None => false,
        }
    }

    /// Adds a context document, unless it is already referenced.
    pub fn reference(&mut self, iri: impl Into<String>) -> bool {
        let iri = iri.into();
        let known = self.items().any(|entry| match entry {
            ContextItem::Reference(existing) => {
                terms::normalize(existing) == terms::normalize(&iri)
            }
            ContextItem::Definitions(_) => false,
        });
        if known {
            return false;
        }
        self.push(ContextItem::Reference(iri));
        true
    }

    /// Defines `term` inline, as crates do for terms no published context
    /// covers.
    pub fn define(&mut self, term: impl Into<String>, iri: impl Into<String>) {
        let (term, iri) = (term.into(), iri.into());
        if let Some(ContextItem::Definitions(definitions)) = self.last_mut() {
            definitions.insert(term, iri);
            return;
        }
        self.push(ContextItem::Definitions(IndexMap::from([(term, iri)])));
    }

    fn last_mut(&mut self) -> Option<&mut ContextItem> {
        match &mut self.0 {
            OneOrMany::One(item) => Some(item),
            OneOrMany::Many(items) => items.last_mut(),
        }
    }

    fn push(&mut self, item: ContextItem) {
        let placeholder = OneOrMany::Many(Vec::new());
        let mut items = std::mem::replace(&mut self.0, placeholder).into_many();
        items.push(item);
        self.0 = OneOrMany::Many(items);
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
