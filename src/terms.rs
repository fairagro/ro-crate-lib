use crate::constants::{
    BASE_TERMS, TEST_CONTEXT, TEST_TERMS, WORKFLOW_RUN_CONTEXT, WORKFLOW_RUN_TERMS,
};

#[must_use]
pub fn context_defines(iri: &str, term: &str) -> bool {
    match definitions_of(iri) {
        Some(terms) => terms.iter().any(|(name, _)| *name == term),
        None => is_base_context(iri) && BASE_TERMS.contains(&term),
    }
}

/// The context document that defines `term`, for crates that need one added.
#[must_use]
pub fn defining_context(term: &str) -> Option<&'static str> {
    if WORKFLOW_RUN_TERMS.iter().any(|(name, _)| *name == term) {
        Some(WORKFLOW_RUN_CONTEXT)
    } else if TEST_TERMS.iter().any(|(name, _)| *name == term) {
        Some(TEST_CONTEXT)
    } else {
        None
    }
}
/// The IRI a term expands to, for crates that inline definitions instead of
/// referencing the context document.
#[must_use]
pub fn expansion(term: &str) -> Option<&'static str> {
    WORKFLOW_RUN_TERMS
        .iter()
        .chain(TEST_TERMS)
        .find(|(name, _)| *name == term)
        .map(|(_, iri)| *iri)
}

fn definitions_of(iri: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match normalize(iri) {
        i if i == normalize(WORKFLOW_RUN_CONTEXT) => Some(WORKFLOW_RUN_TERMS),
        i if i == normalize(TEST_CONTEXT) => Some(TEST_TERMS),
        _ => None,
    }
}

/// Any `https://w3id.org/ro/crate/<version>/context`.
fn is_base_context(iri: &str) -> bool {
    base_context_version(iri).is_some()
}

/// The RO-Crate version a base context document is for.
#[must_use]
pub fn base_context_version(iri: &str) -> Option<&str> {
    normalize(iri)
        .strip_prefix("w3id.org/ro/crate/")?
        .strip_suffix("/context")
}

/// Ignores the `http`/`https` split and a trailing slash, both of which show
/// up in published crates.
pub(crate) fn normalize(iri: &str) -> &str {
    iri.strip_prefix("http://")
        .or_else(|| iri.strip_prefix("https://"))
        .unwrap_or(iri)
        .trim_end_matches('/')
}
