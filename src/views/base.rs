use crate::{
    define_views,
    graph::node::{GraphNode, Value},
    views::View,
};

define_views! {
    RootDataset {
        types: ["Dataset"],
        terms: ["hasPart", "mainEntity"],
    }

    Person {
        types: ["Person"],
        terms: ["name"],
    }

    Organization {
        types: ["Organization"],
        terms: ["name"],
    }

    PropertyValue {
        types: ["PropertyValue"],
        terms: ["value"],
    }
}

impl<'a> RootDataset<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn description(&self) -> Option<&'a str> {
        self.text("description")
    }

    pub fn date_published(&self) -> Option<&'a str> {
        self.text("datePublished")
    }

    /// `license` is a text IRI as often as it is a reference.
    pub fn license(&self) -> Option<&'a str> {
        self.text("license")
            .or_else(|| self.ref_ids("license").into_iter().next())
    }

    pub fn parts(&self) -> Vec<&'a GraphNode> {
        self.nodes("hasPart")
    }

    pub fn authors(&self) -> Vec<Person<'a>> {
        self.resolve_all("author")
    }

    pub fn publisher(&self) -> Option<Organization<'a>> {
        self.resolve("publisher")
    }

    pub fn mentions(&self) -> Vec<&'a GraphNode> {
        self.nodes("mentions")
    }
}

impl<'a> Person<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn identifier(&self) -> Option<&'a str> {
        self.text("identifier")
            .or_else(|| self.ref_ids("identifier").into_iter().next())
    }

    pub fn affiliations(&self) -> Vec<Organization<'a>> {
        self.resolve_all("affiliation")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> Organization<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    pub fn url(&self) -> Option<&'a str> {
        self.text("url")
            .or_else(|| self.ref_ids("url").into_iter().next())
    }
}

impl<'a> PropertyValue<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.text("name")
    }

    /// Values are published as text, numbers and booleans alike.
    pub fn value(&self) -> Option<&'a Value> {
        self.get("value")
    }

    pub fn text_value(&self) -> Option<&'a str> {
        self.text("value")
    }

    /// What this value is an instance of — a workflow parameter, in a
    /// Workflow Run Crate.
    pub fn example_of_work(&self) -> Option<&'a GraphNode> {
        self.node_at("exampleOfWork")
    }
}
