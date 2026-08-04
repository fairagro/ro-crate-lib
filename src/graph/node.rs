use indexmap::IndexMap;
use oneormany::{OneOrMany, OneOrManyRef};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GraphNode {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub types: OneOrMany<String>,
    #[serde(flatten)]
    pub properties: IndexMap<String, Value>,
}

impl GraphNode {
    pub fn new(id: impl Into<String>, types: impl Into<OneOrMany<String>>) -> Self {
        GraphNode {
            id: id.into(),
            types: types.into(),
            properties: IndexMap::new(),
        }
    }

    #[must_use]
    pub fn has_type(&self, type_name: &str) -> bool {
        self.types.iter().any(|t| t == type_name)
    }

    #[must_use]
    pub fn has_types(&self, type_names: &[&str]) -> bool {
        type_names.iter().all(|t| self.has_type(t))
    }

    #[must_use]
    pub fn get(&self, term: &str) -> Option<&Value> {
        self.properties.get(term)
    }

    pub fn set(&mut self, term: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.properties.insert(term.into(), value.into());
        self
    }

    pub fn push_ref(&mut self, term: impl Into<String>, reference: Reference) -> &mut Self {
        let term = term.into();
        match self.properties.get_mut(&term) {
            Some(Value::List(items)) => items.push(Value::Reference(reference)),
            Some(existing) => {
                let previous = std::mem::replace(existing, Value::Bool(false));
                *existing = Value::List(vec![previous, Value::Reference(reference)]);
            }
            None => {
                self.properties.insert(term, Value::Reference(reference));
            }
        }
        self
    }

    #[must_use]
    pub fn reference(&self) -> Reference {
        Reference::new(self.id.clone())
    }

    pub fn refs(&self, term: &str) -> impl Iterator<Item = &str> {
        self.get(term).into_iter().flat_map(Value::refs)
    }

    #[must_use]
    pub fn text(&self, term: &str) -> Option<&str> {
        self.get(term)?.as_str()
    }

    /// `@id` references and bare strings, for terms crates publish both ways.
    pub fn iris(&self, term: &str) -> impl Iterator<Item = &str> {
        self.get(term)
            .into_iter()
            .flat_map(|value| value.refs().chain(value.strings()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Number(serde_json::Number),
    Text(String),
    List(Vec<Value>),
    Reference(Reference),
    Object(IndexMap<String, Value>),
}

impl Value {
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The `@id` of a single reference value.
    #[must_use]
    pub fn as_ref_id(&self) -> Option<&str> {
        match self {
            Value::Reference(r) => Some(&r.id),
            _ => None,
        }
    }

    #[must_use]
    pub fn items(&self) -> OneOrManyRef<'_, Value> {
        match self {
            Value::List(items) => OneOrManyRef::new_from_slice(items),
            single => OneOrManyRef::new_from_scalar(single),
        }
    }

    pub fn refs(&self) -> impl Iterator<Item = &str> {
        self.items().filter_map(Value::as_ref_id)
    }

    pub fn strings(&self) -> impl Iterator<Item = &str> {
        self.items().filter_map(Value::as_str)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n.into())
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Number(n.into())
    }
}

impl From<Reference> for Value {
    fn from(r: Reference) -> Self {
        Value::Reference(r)
    }
}

impl From<Vec<Value>> for Value {
    fn from(values: Vec<Value>) -> Self {
        match <[Value; 1]>::try_from(values) {
            Ok([single]) => single,
            Err(many) => Value::List(many),
        }
    }
}

impl From<Vec<Reference>> for Value {
    fn from(refs: Vec<Reference>) -> Self {
        match <[Reference; 1]>::try_from(refs) {
            Ok([single]) => Value::Reference(single),
            Err(many) => Value::List(many.into_iter().map(Value::Reference).collect()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    #[serde(rename = "@id")]
    pub id: String,
}

impl Reference {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl<T: Into<String>> From<T> for Reference {
    fn from(id: T) -> Self {
        Self::new(id)
    }
}
